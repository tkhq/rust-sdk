#!/usr/bin/env python3
"""Run tvc-deployments evals.json cases through `claude -p` and grade the assertions.

Safety: Bash/Write/Edit/Web tools are disallowed for the evaluated agent, so it can
only read the skill and answer. No tvc command is ever executed.
"""
import argparse, json, os, re, subprocess, sys, time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

DISALLOWED = ["Bash", "Edit", "Write", "MultiEdit", "NotebookEdit", "WebFetch", "WebSearch", "Agent", "Task"]

def run_case(case, sandbox, model, max_turns, timeout):
    cmd = ["claude", "-p", case["prompt"], "--output-format", "stream-json", "--verbose",
           "--max-turns", str(max_turns), "--no-session-persistence", "--strict-mcp-config",
           "--setting-sources", "project", "--disallowedTools", *DISALLOWED]
    if model:
        cmd += ["--model", model]
    env = {k: v for k, v in os.environ.items() if k != "CLAUDECODE"}
    t0 = time.time()
    proc = subprocess.run(cmd, cwd=sandbox, env=env, stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=timeout)
    dur = time.time() - t0
    texts, tools, final, skill_used, files_read, errors, tool_results = [], [], "", False, [], [], []
    for line in proc.stdout.splitlines():
        try:
            ev = json.loads(line)
        except json.JSONDecodeError:
            continue
        if ev.get("type") == "assistant":
            for c in ev.get("message", {}).get("content", []):
                if c.get("type") == "text":
                    texts.append(c["text"])
                elif c.get("type") == "tool_use":
                    tools.append({"name": c.get("name"), "input": c.get("input")})
                    if c.get("name") == "Skill":
                        skill_used = True
                    if c.get("name") == "Read":
                        files_read.append(str(c.get("input", {}).get("file_path", "")))
        elif ev.get("type") == "user":
            for c in ev.get("message", {}).get("content", []) if isinstance(ev.get("message", {}).get("content"), list) else []:
                if c.get("type") == "tool_result":
                    body = c.get("content")
                    if isinstance(body, list):
                        body = " ".join(x.get("text", "") for x in body if isinstance(x, dict))
                    tool_results.append({"is_error": c.get("is_error", False), "text": str(body)[:400]})
        elif ev.get("type") == "result":
            final = ev.get("result", "") or ""
            if ev.get("is_error"):
                errors.append(ev.get("result"))
    full = "\n".join(texts + [json.dumps(t["input"]) for t in tools])
    return {"stdout_ok": proc.returncode == 0, "stderr": proc.stderr[-2000:], "duration_s": round(dur, 1),
            "final": final, "full": full, "tools": [t["name"] for t in tools], "skill_used": skill_used,
            "files_read": files_read, "errors": errors, "tool_results": tool_results}

def grade(case, run):
    target = run["full"] if case.get("gradeFullResponse") else run["final"]
    results = []
    for a in case["assertions"]:
        t = a["type"]
        flags = 0
        for ch in a.get("flags", ""):
            flags |= {"m": re.M, "i": re.I, "s": re.S}.get(ch, 0)
        if t == "contains":
            ok = a["value"] in target
        elif t == "not_contains":
            ok = a["value"] not in target
        elif t == "regex":
            ok = re.search(a.get("value", a.get("pattern", "")), target, flags) is not None
        elif t == "not_regex":
            ok = re.search(a.get("pattern", a.get("value", "")), target, flags) is None
        else:
            ok = False
        results.append({"type": t, "value": a.get("value", a.get("pattern")), "passed": ok, "comment": a.get("comment", "")})
    return results

def record(case, run_no, run):
    """Grade one run. A run whose subprocess failed or whose result event was an error
    counts every assertion as failed: a partial response may contain the expected
    keywords, and the negative assertions pass trivially on an empty one."""
    g = grade(case, run)
    run_ok = run["stdout_ok"] and not run["errors"]
    if not run_ok:
        for a in g:
            a["passed"] = False
            a["comment"] = "run failed; " + (a.get("comment") or "")
    return {"id": case["id"], "run": run_no, "run_ok": run_ok, "passed": sum(x["passed"] for x in g), "total": len(g),
            "assertions": g, "skill_used": run["skill_used"], "tools": run["tools"], "files_read": run["files_read"],
            "duration_s": run["duration_s"], "errors": run["errors"], "stderr": run["stderr"]}

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--evals", required=True)
    ap.add_argument("--sandbox", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--model", default=None)
    ap.add_argument("--ids", default="")
    ap.add_argument("--runs", type=int, default=1)
    ap.add_argument("--concurrency", type=int, default=3)
    ap.add_argument("--max-turns", type=int, default=12)
    ap.add_argument("--timeout", type=int, default=600)
    args = ap.parse_args()
    data = json.loads(Path(args.evals).read_text())
    cases = data["evals"]
    if args.ids:
        want = {int(x) for x in args.ids.split(",")}
        cases = [c for c in cases if c["id"] in want]
    out = Path(args.out); out.mkdir(parents=True, exist_ok=True)
    jobs = [(c, r) for c in cases for r in range(1, args.runs + 1)]
    summary = []
    with ThreadPoolExecutor(max_workers=args.concurrency) as ex:
        futs = {ex.submit(run_case, c, args.sandbox, args.model, args.max_turns, args.timeout): (c, r) for c, r in jobs}
        for fut in as_completed(futs):
            c, r = futs[fut]
            try:
                run = fut.result()
            except Exception as e:
                run = {"stdout_ok": False, "stderr": str(e), "duration_s": 0, "final": "", "full": "", "tools": [], "skill_used": False, "files_read": [], "errors": [str(e)]}
            rec = record(c, r, run)
            (out / f"eval-{c['id']}-run{r}.json").write_text(json.dumps({**rec, "final": run["final"], "full": run["full"], "tool_results": run.get("tool_results", [])}, indent=2))
            summary.append(rec)
            status = "ERROR" if not rec["run_ok"] else "PASS" if rec["passed"] == rec["total"] else "FAIL"
            print(f"[{status}] eval {c['id']:>2} run {r}: {rec['passed']}/{rec['total']} skill_used={rec['skill_used']} tools={rec['tools']} {rec['duration_s']}s", flush=True)
            for a in rec["assertions"]:
                if not a["passed"]:
                    print(f"         x {a['type']} {a['value']!r} {('- ' + a['comment']) if a['comment'] else ''}", flush=True)
    summary.sort(key=lambda x: (x["id"], x["run"]))
    (out / "summary.json").write_text(json.dumps(summary, indent=2))
    tot = sum(r["total"] for r in summary); ok = sum(r["passed"] for r in summary)
    print(f"\nassertions passed: {ok}/{tot}; cases fully green: {sum(1 for r in summary if r['passed']==r['total'])}/{len(summary)}")

if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Run tvc-deployments evals.json cases through `claude -p` and grade the assertions.

Safety: Bash/Write/Edit/Web tools are disallowed for the evaluated agent, so it can
only read the skill and answer. No tvc command is ever executed.

Usage:
    python3 run_evals.py --evals evals.json --sandbox <project with .claude/skills/tvc-deployments> \
        --baseline-sandbox <empty project> --out <results dir> --model sonnet

Each case runs --runs times (default 3) because the agent is not deterministic: a single
pass proves the skill produced the right answer once, not that it reliably does. The suite
is green only when every run of every case passes. With --baseline-sandbox the same cases
also run without the skill; a case the baseline passes in every run does not discriminate
and is flagged. The results directory gets per-run JSON, summary.json, meta.json (model,
Claude Code version, commit, skill fingerprint) and report.md, a table ready for a PR.
"""
import argparse, datetime, hashlib, json, os, re, subprocess, sys, time
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
    texts, tools, final, skill_used, files_read, errors, tool_results, model_seen = [], [], "", False, [], [], [], None
    for line in proc.stdout.splitlines():
        try:
            ev = json.loads(line)
        except json.JSONDecodeError:
            continue
        if ev.get("type") == "system" and ev.get("subtype") == "init":
            model_seen = ev.get("model")
        elif ev.get("type") == "assistant":
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
            "files_read": files_read, "errors": errors, "tool_results": tool_results, "model": model_seen}

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
            "duration_s": run["duration_s"], "errors": run["errors"], "stderr": run["stderr"], "model": run.get("model")}

def green(rec):
    return rec["run_ok"] and rec["passed"] == rec["total"]

def run_suite(label, cases, sandbox, out, args):
    """Run every case --runs times in one sandbox. Returns the list of graded runs."""
    out.mkdir(parents=True, exist_ok=True)
    jobs = [(c, r) for c in cases for r in range(1, args.runs + 1)]
    summary, lines = [], []
    with ThreadPoolExecutor(max_workers=args.concurrency) as ex:
        futs = {ex.submit(run_case, c, sandbox, args.model, args.max_turns, args.timeout): (c, r) for c, r in jobs}
        for fut in as_completed(futs):
            c, r = futs[fut]
            try:
                run = fut.result()
            except Exception as e:
                run = {"stdout_ok": False, "stderr": str(e), "duration_s": 0, "final": "", "full": "", "tools": [], "skill_used": False, "files_read": [], "errors": [str(e)], "model": None}
            rec = record(c, r, run)
            (out / f"eval-{c['id']}-run{r}.json").write_text(json.dumps({**rec, "final": run["final"], "full": run["full"], "tool_results": run.get("tool_results", [])}, indent=2))
            summary.append(rec)
            status = "ERROR" if not rec["run_ok"] else "PASS" if green(rec) else "FAIL"
            line = f"[{status}] {label} eval {c['id']:>2} run {r}: {rec['passed']}/{rec['total']} skill_used={rec['skill_used']} tools={rec['tools']} {rec['duration_s']}s"
            print(line, flush=True)
            lines.append(line)
            for a in rec["assertions"]:
                if not a["passed"]:
                    print(f"         x {a['type']} {a['value']!r} {('- ' + a['comment']) if a['comment'] else ''}", flush=True)
    summary.sort(key=lambda x: (x["id"], x["run"]))
    lines.sort(key=lambda s: tuple(int(x) for x in re.search(r"eval\s+(\d+) run (\d+)", s).groups()))
    (out / "summary.json").write_text(json.dumps(summary, indent=2))
    (out / "runs.log").write_text("\n".join(lines) + "\n")
    return summary

def skill_fingerprint(sandbox):
    """Hash of every file under the sandbox's .claude/skills, so a report says which skill text it graded."""
    root = Path(sandbox) / ".claude" / "skills"
    h, n = hashlib.sha256(), 0
    for p in sorted(root.rglob("*")) if root.is_dir() else []:
        if p.is_file():
            h.update(p.relative_to(root).as_posix().encode()); h.update(p.read_bytes()); n += 1
    return {"files": n, "sha256": h.hexdigest()[:16] if n else None}

def best_effort(cmd):
    try:
        return subprocess.run(cmd, capture_output=True, text=True, timeout=30).stdout.strip() or None
    except Exception:
        return None

def per_case(cases, summary, runs):
    """Per case: how many runs were fully green and the assertion pass rate across runs."""
    table = {}
    for c in cases:
        recs = [r for r in summary if r["id"] == c["id"]]
        table[c["id"]] = {"runs_green": sum(green(r) for r in recs), "runs": runs,
                          "passed": sum(r["passed"] for r in recs), "total": sum(r["total"] for r in recs)}
    return table

def write_report(out, meta, cases, with_s, base_s, args):
    w = per_case(cases, with_s, args.runs)
    b = per_case(cases, base_s, args.runs) if base_s is not None else None
    hdr = "| eval | with skill: runs green | assertions | baseline: runs green | assertions | discriminates |" if b else "| eval | runs green | assertions |"
    sep = "|---|---|---|---|---|---|" if b else "|---|---|---|"
    rows, weak = [], []
    for c in cases:
        i = c["id"]; x = w[i]
        if b:
            y = b[i]
            disc = y["runs_green"] < args.runs
            if not disc:
                weak.append(i)
            rows.append(f"| {i} | {x['runs_green']}/{x['runs']} | {x['passed']}/{x['total']} | {y['runs_green']}/{y['runs']} | {y['passed']}/{y['total']} | {'yes' if disc else 'NO'} |")
        else:
            rows.append(f"| {i} | {x['runs_green']}/{x['runs']} | {x['passed']}/{x['total']} |")
    def totals(s):
        return (sum(green(r) for r in s), len(s), sum(r["passed"] for r in s), sum(r["total"] for r in s))
    wg, wn, wp, wt = totals(with_s)
    md = [f"## Eval report: {meta['skill_name']}", "",
          f"- Date: {meta['date']}  ",
          f"- Model: requested `{meta['model_requested']}`, observed `{', '.join(meta['models_observed']) or 'unknown'}`  ",
          f"- Claude Code: `{meta['claude_version']}`  ",
          f"- Commit: `{meta['git_head'] or 'unknown'}`; skill fingerprint `{meta['skill']['sha256']}` ({meta['skill']['files']} files)  ",
          f"- Runs per case: {args.runs}; cases: {len(cases)}; sandboxed agent with {', '.join(DISALLOWED)} disallowed", "",
          hdr, sep, *rows, "",
          f"**With skill:** {wg}/{wn} runs green, {wp}/{wt} assertions."]
    if b:
        bg, bn, bp, bt = totals(base_s)
        md.append(f"**Without skill:** {bg}/{bn} runs green, {bp}/{bt} assertions.")
        md.append(("**Non-discriminating cases (baseline passed every run):** " + ", ".join(map(str, weak))) if weak else "**Every case discriminates:** the baseline failed at least one run of each.")
    md += ["", "<details><summary>Per-run lines, with skill</summary>", "", "```", (out / "runs.log").read_text().rstrip(), "```", "</details>"]
    if b:
        md += ["", "<details><summary>Per-run lines, without skill</summary>", "", "```", (out / "baseline" / "runs.log").read_text().rstrip(), "```", "</details>"]
    (out / "report.md").write_text("\n".join(md) + "\n")
    return weak

def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--evals", required=True)
    ap.add_argument("--sandbox", required=True, help="project dir whose .claude/skills holds the skill under test")
    ap.add_argument("--baseline-sandbox", default=None, help="empty project dir; runs the same cases without the skill and reports lift")
    ap.add_argument("--out", required=True)
    ap.add_argument("--model", default=None)
    ap.add_argument("--ids", default="")
    ap.add_argument("--runs", type=int, default=3, help="runs per case; every run must pass (default 3)")
    ap.add_argument("--concurrency", type=int, default=3)
    ap.add_argument("--max-turns", type=int, default=12)
    ap.add_argument("--timeout", type=int, default=600)
    args = ap.parse_args()
    data = json.loads(Path(args.evals).read_text())
    cases = data["evals"]
    if args.ids:
        want = {int(x) for x in args.ids.split(",")}
        cases = [c for c in cases if c["id"] in want]
    out = Path(args.out)
    with_s = run_suite("with   ", cases, args.sandbox, out, args)
    base_s = run_suite("without", cases, args.baseline_sandbox, out / "baseline", args) if args.baseline_sandbox else None
    meta = {"skill_name": data.get("skill_name", "?"), "date": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d %H:%M UTC"),
            "model_requested": args.model or "default", "models_observed": sorted({r["model"] for r in with_s + (base_s or []) if r.get("model")}),
            "claude_version": best_effort(["claude", "--version"]), "git_head": best_effort(["git", "-C", str(Path(args.evals).resolve().parent), "rev-parse", "--short", "HEAD"]),
            "skill": skill_fingerprint(args.sandbox), "baseline_skill": skill_fingerprint(args.baseline_sandbox) if args.baseline_sandbox else None,
            "runs": args.runs, "ids": [c["id"] for c in cases]}
    (out / "meta.json").write_text(json.dumps(meta, indent=2))
    weak = write_report(out, meta, cases, with_s, base_s, args)
    tot = sum(r["total"] for r in with_s); ok = sum(r["passed"] for r in with_s); g = sum(green(r) for r in with_s)
    print(f"\nwith skill: assertions {ok}/{tot}; runs green {g}/{len(with_s)}")
    if base_s is not None:
        bt = sum(r["total"] for r in base_s); bo = sum(r["passed"] for r in base_s); bg = sum(green(r) for r in base_s)
        print(f"without skill: assertions {bo}/{bt}; runs green {bg}/{len(base_s)}")
        if weak:
            print(f"WARNING: baseline passed every run of eval(s) {weak}; these cases do not discriminate", flush=True)
    print(f"report: {out / 'report.md'}")
    # Red result, red exit: CI must not accept failed evals or runs that never completed.
    sys.exit(1 if not with_s or g != len(with_s) else 0)

if __name__ == "__main__":
    main()

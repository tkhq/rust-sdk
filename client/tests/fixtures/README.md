`signature_digest_vectors.json` contains TKMS signature-digest compatibility
vectors generated independently of this SDK. The cases cover both supported
signing algorithms and changes to each original field. The receipt tests check these
digests through the same stamp decoder used on SDK requests.

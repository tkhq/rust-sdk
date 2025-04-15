# Proto

This folder contains the Turnkey proto files. They're synced to this repo when an update is required, from Turnkey's internal monorepo.

To sync manually, if you have the two repos side-by-side and you are at the root of rust-sdk:
* `cp -r ../mono/proto/external proto`
* `cp -r ../mono/proto/immutable proto`
* `cp -r ../mono/proto/vendor proto`
* `cp -r ../mono/proto/services/coordinator/public proto/services/coordinator`

# Why does `/dev/null` exist?

The `/dev/null` service exists to allow engineers to [blackbox](https://en.wikipedia.org/wiki/Black-box_testing) test their service dependencies.

As services mature and grow, they end up taking on many dependencies. Often those dependencies don't have a "sandbox" account that acts correctly. That's where the `/dev/null` service comes in. It pretends to be that service, and will respond with whatever text/json you want.

This means you can record some normal API interactions, then _save_ those into a config file, and `/dev/null` will respond with that data.

## What's the catch?

The data is provided by you, and not verified in any way (other than json is actually json). If the API changes, you will need to update the config, or you will run against incorrect configuration. Most stable services don't remove fields willy-nilly.

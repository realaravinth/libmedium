<div align="center">
<h1> LibMedium </h1>
<p>

**Privacy-focused proxy for medium.com**

</p>

[![Awesome Humane Tech](https://raw.githubusercontent.com/humanetech-community/awesome-humane-tech/main/humane-tech-badge.svg?sanitize=true)](https://github.com/humanetech-community/awesome-humane-tech)
[![status-badge](https://ci.batsense.net/api/badges/realaravinth/libmedium/status.svg)](https://ci.batsense.net/realaravinth/libmedium)
[![codecov](https://codecov.io/gh/realaravinth/libmedium/branch/master/graph/badge.svg)](https://codecov.io/gh/realaravinth/libmedium)

</div>

## Status

Usable. Should you run into a `HTTP 500 Internal Server Error`, please
file a bug report with the URL of the post you were trying to read and
the git commit hash of the build. Git commit hash can be obtained from
[/api/v1/meta/build](https://libmedium.batsense.net/api/v1/meta/build).

This proxy works by interacting with Medium's undocumented(probably
private but unauthenticated) API. So I've had to make assumptions and
tweak API schematics as I run into errors.

## Features

-   [x] proxy images
-   [x] proxy GitHub gists
-   [x] render posts
-   [x] syntax highlighting for gists
-   [ ] user pages(WIP)
-   [ ] RSS feeds

## Why?

Knowledge is the true wealth of humanity. If it weren't for our
ancestors, who chose to pass down their knowledge and experiences, we
would still be a primitive species. Whatever advancement that we as
a species have achieved is because we chose to share information.

To put a paywall on knowledge like that is both obscene and goes against
the very nature of humanity.

It is possible to run a sustainable publication business while still
respecting freedom. [LWN.net](https://lwn.net) is one of my favourite
publications that has been around forever. So it is possible. I hope
medium.com comes up with other, non-harmful ways to run a sustainable
business.

## Instances

| Instance                                                                  | Country | Provider   | Host                                   |
| ------------------------------------------------------------------------- | ------- | ---------- | -------------------------------------- |
| https://libmedium.batsense.net                                            | India   | Airtel     | @realaravinth                          |
| https://md.vern.cc                                                        | US      | Hetzner    | [~vern](https://vern.cc)               |
| http://md.vernccvbvyi5qhfzyqengccj7lkove6bjot2xhh5kajhwvidqafczrad.onion/ | N/A     | Hetzner    | [~vern](https://vern.cc)               |
| http://vernaqj2qr2pijpgvf3od6ssc3ulz3nv52gwr3hba5l6humuzmgq.b32.i2p/      | N/A     | Hetzner    | [~vern](https://vern.cc)               |
| https://medium.hostux.net                                                 | France  | Gandi      | [hostux](https://hostux.net)           |
| https://r.sudovanilla.org                                                 | US      | Selfhosted | [SudoVanilla](https://sudovanilla.org) |
| https://libmedium.ducks.party                                             | DE      | Datalix    | [ducks.party](https://ducks.party)     |
## Deploy

1. Grab [`./config/default.toml`](./config/default.toml) and make
   necessary changes

2. AMD64 pre-compiled images are available on DockerHub.

```
docker run -d \
  -v ./config/default.toml:/etc/libmedium/config.toml \
  -p 8082:7000 \
  --restart always \
  --name libmedium \
  realaravinth/libmedium
```

If you are on a different architecture, run make docker and then run the
above command.

```
make docker
```

---

Inspired by [Scribe - An Alternative Medium Frontend](https://sr.ht/~edwardloveall/scribe)

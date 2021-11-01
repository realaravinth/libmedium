<div align="center">
<h1> LibMedium </h1>
<p>

**Privacy-focused proxy for medium.com**

</p>

[![Build](https://github.com/realaravinth/libmedium/actions/workflows/linux.yml/badge.svg)](https://github.com/realaravinth/libmedium/actions/workflows/linux.yml)
[![dependency status](https://deps.rs/repo/github/realaravinth/libmedium/status.svg)](https://deps.rs/repo/github/realaravinth/libmedium)
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

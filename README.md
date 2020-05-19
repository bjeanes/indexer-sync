# Indexer Sync

Automatically configure torrent and usenet indexers in Sonarr, Radarr,
Lidarr, CouchPotato, Sickbeard, SickRage, etc based on the definitions in
applications like Jackett, Cardigann, NZBHydra2, etc.

You should be able to add a new indexer to Jackett et al and with one run of
this tool have a new indexer entry in Sonarr et al. Likewise, you should be
able to rotate your API token in Jackett et al and have it update all the
indexers defined in Sonarr et al.

## Install

You can download pre-built binaries and view release notes in GitHub releases.

* [latest release](https://github.com/bjeanes/indexer-sync/releases/latest)
* [`unstable` release](https://github.com/bjeanes/indexer-sync/releases/tag/unstable)
  \- built from `master` when tests pass

You can also pull [releases from DockerHub](https://hub.docker.com/repository/docker/bjeanes/indexer-sync):

```sh-session
$ docker pull bjeanes/indexer-sync:unstable # built from master when tests pass
$ docker pull bjeanes/indexer-sync:latest   # built from latest tagged release
$ docker pull bjeanes/indexer-sync:v0.1     # built from specified tagged release
```

## Usage

This is very early work-in-progress so the following is aspirational:

``` sh-session
$ indexer-sync --help
indexer-sync 0.2.0
Bo Jeanes <me@bjeanes.com>
At least one [src] and at least one [dst] must be specified in order to sync

USAGE:
    indexer-sync [OPTIONS] <--jackett <jackett>> <--sonarr <sonarr>> [INDEXER]...

ARGS:
    <INDEXER>...    Provide indexers that you want to update. These values will be case-insensitively substring
                    matched against indexer/tracker names. Only those which match will be synced. If not provided,
                    all discovered indexers will be synced

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --interval <interval>    The interval (in seconds) between sync runs. Syncer will run once and exit if this is
                                 not provided [env: SYNC_INTERVAL=]
    -J, --jackett <jackett>      [src] URL to Jackett instance from where indexers should be sourced Basic Auth
                                 credentials will be extracted and used as admin password [env: SYNC_JACKETT_URL=]
    -S, --sonarr <sonarr>        [dst] URL to Sonarr instance where indexers should be updated. Encoded Basic Auth
                                 credentials will be extracted and used as the API token [env: SYNC_SONARR_URL=]

$ indexer-sync --jackett http://$JACKETT_ADMIN_PW@jackett-instance:1234 --sonarr http://$SONARR_API_KEY@sonarr-instance:5678/
 INFO  indexer_sync > Fetching indexers from Jackett
 INFO  indexer_sync > Updating indexers in Sonarr
 INFO  indexer_sync::destination::sonarr > Updating ETTV {jackett:ettv} in Sonarr (id: 28)
 INFO  indexer_sync::destination::sonarr > Updating RARBG {jackett:rarbg} in Sonarr (id: 35)
...
```

## Contributing

Contributions are welcome.

The domain model is a first pass as I'm learning the APIs of these tools and
the required fields and how they vary based on indexer types etc.

This is one of my first Rust projects so the code will be rough. There's
already ways I know it is rough but I haven't got over the learning curve and
there are bound to be ways in which I don't yet know it's rough.

## Planned features (rough priority order)

* [x] Pull indexer definitions from Jackett
* [x] Add/update indexers in Sonarr
* [ ] Add/update indexers in Radarr
* [ ] Allowing the specification of seed criteria

   In particular, allow setting it separately for public vs private trackers
* [x] Long-running mode where it polls and updates definitions on a defined interval
* [x] Optional arguments to filter indexer names to sync

   This would be useful if you wanted to set up separate regular syncs but
   with different indexers to different media managers. For instance:

   ```sh-session
   $ indexer-sync --jackett $JACKET_URL --sonarr $SONARR_URL idope tpb rarbg
   $ indexer-sync --jackett $JACKET_URL --radarr $RADARR_URL yts tpb rarbg
   ```
* [ ] Explicit specification of category/capability IDs for each media type.

   For example:

   ```sh-session
   $ indexer-sync --jackett $JACKET_URL --sonarr $SONARR_URL --tv-categories=5000,5030,5040
   ```

   It would filter the categories each indexer supports to the ones from that
   list when passed to the media manager.
* [ ] Docker image and `docker-compose.yml` example so it can be set-and-forget
* [ ] Pull indexer definitions from NZBHydra2
* [ ] Add/update indexers in Lidarr
* After that, I'd be happy to grow this tool to support the following, but I do not personally use these:
   * [ ] CouchPotato
   * [ ] Sickbeard
   * [ ] Sickrage
   * [ ] Cardigann

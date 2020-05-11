# Indexer Sync

Automatically configure torrent and usenet indexers in Sonarr, Radarr,
Lidarr, CouchPotato, Sickbeard, SickRage, etc based on the definitions in
applications like Jackett, Cardigann, NZBHydra2, etc.

You should be able to add a new indexer to Jackett et al and with one run of
this tool have a new indexer entry in Sonarr et al. Likewise, you should be
able to rotate your API token in Jackett et al and have it update all the
indexers defined in Sonarr et al.

## Usage

This is very early work-in-progress so the following is aspirational:

``` sh-session
$ indexer-sync --help
indexer-sync 0.1.0
Bo Jeanes <me@bjeanes.com>
At least one [src] and at least one [dst] must be specified in order to sync

USAGE:
    indexer-sync [OPTIONS] <--jackett <jackett>> <--sonarr <sonarr>|--radarr <radarr>>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --jackett <jackett>    [src] URL to Jackett instance from where indexers should be sourced. Basic Auth
                               credentials will be extracted and used as admin password [env: SYNC_JACKETT_URL=]
        --radarr <radarr>      [dst] URL to Radarr instance where indexers should be updated. Basic Auth
                               credentials will be extracted and used as the API token [env: SYNC_RADARR_URL=]
        --sonarr <sonarr>      [dst] URL to Sonarr instance where indexers should be updated. Basic Auth

$ indexer-sync --jackett http://$JACKETT_ADMIN_PW@jackett-instance:1234 --sonarr http://$SONARR_API_KEY@sonarr-instance:5678/
Adding ETTV to Sonarr... [done]
Updating RarBG in Sonarr... [done]
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
* [ ] Long-running mode where it polls and updates definitions on a defined interval
* [ ] Docker image and `docker-compose.yml` example so it can be set-and-forget
* [ ] Pull indexer definitions from NZBHydra2
* [ ] Add/update indexers in Lidarr
* After that, I'd be happy to grow this tool to support the following, but I do not personally use these:
   * [ ] CouchPotato
   * [ ] Sickbeard
   * [ ] Sickrage
   * [ ] Cardigann
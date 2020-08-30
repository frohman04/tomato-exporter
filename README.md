# tomato-exporter

[node_exporter](https://github.com/prometheus/node_exporter) work-alike for routers running 
[FreshTomato](https://freshtomato.org/) firmware on their routers.  This exporter is unique in that it does *not* run on
the router, but on another machine that is able to access the router's admin interface.  The exporter provides a subset
of the metrics provided by `node_exporter`, but using the same metric names and formats, so that any pre-existing 
`node_exporter` dashboard you have in Grafana can be populated by data from this exporter.

## Configuration

See example.yaml for example configuration file that includes all available properties and documentation for each.

## How does it work?

Since `tomato-exporter` doesn't run directly on the target system, it has to make due with what it has available to it:
the FreshTomato web UI.  It turns out that the dynamic parts of the UI are powered by HTTP APIs (though not RESTful)
that we can access using Basic HTTP Auth and the `_http_id`.  We're able to then abuse the endpoint that backs the web
UI's root console to execute shell commands directly on the router.  While this is nowhere near as fast as executing the
commands via a remote shell or as flexible as providing code to run directly on the router, it does give us access to
the `/proc` filesystem as well as a bevy of built-in shell commands and bundled programs.

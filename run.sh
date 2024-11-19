docker run -v ./docker/output:/output \
    -v ./docker/paths:/paths \
    -it --rm --name cc-dl \
    ghcr.io/neodyland/cc-downloader-rs
docker run -v ./docker/assets:/assets \
    -v ./docker/output:/output \
    -v ./docker/paths:/paths \
    -it --rm --name cc-dl \
    googlefan25/cc-dl
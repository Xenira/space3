#!/bin/bash
set -e

##################################################
# General
##################################################
docker build . --target chef -t space-cubed/chef --cache-from=localhost:5000/space-cubed/chef --build-arg BUILDKIT_INLINE_CACHE=1
docker tag space-cubed/chef localhost:5000/space-cubed/chef && docker push localhost:5003/space-cubed/chef || true

##################################################
# Client
##################################################
docker build . --target trunk -t space-cubed/trunk --cache-from=localhost:5000/space-cubed/trunk --build-arg BUILDKIT_INLINE_CACHE=1
docker tag space-cubed/trunk localhost:5000/space-cubed/trunk && docker push localhost:5000/space-cubed/trunk || true

docker build . --target planner-client -t space-cubed/planner-client --cache-from=localhost:5000/space-cubed/planner-client --build-arg BUILDKIT_INLINE_CACHE=1
docker tag space-cubed/planner-client localhost:5000/space-cubed/planner-client && docker push localhost:5000/space-cubed/planner-client || true

docker build . --target builder-client -t space-cubed/builder-client --build-arg BASE_URL="$BASE_URL" --cache-from=localhost:5000/space-cubed/builder-client --build-arg BUILDKIT_INLINE_CACHE=1
docker tag space-cubed/builder-client localhost:5000/space-cubed/builder-client && docker push localhost:5000/space-cubed/builder-client || true

##################################################
# Server
##################################################
docker build . --target planner-server -t space-cubed/planner-server --cache-from=localhost:5000/space-cubed/planner-server --build-arg BUILDKIT_INLINE_CACHE=1
docker tag space-cubed/planner-server localhost:5000/space-cubed/planner-server && docker push localhost:5000/space-cubed/planner-server || true

docker build . --target builder-server -t space-cubed/builder-server --cache-from=localhost:5000/space-cubed/builder-server --build-arg BUILDKIT_INLINE_CACHE=1
docker tag space-cubed/builder-server localhost:5000/space-cubed/builder-server && docker push localhost:5000/space-cubed/builder-server || true

docker build . --target server -t space-cubed/web --cache-from=localhost:5000/space-cubed/web --build-arg BUILDKIT_INLINE_CACHE=1
docker tag space-cubed/web localhost:5000/space-cubed/web && docker push localhost:5000/space-cubed/web || true

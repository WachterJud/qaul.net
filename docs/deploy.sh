#!/bin/sh

# build and upload the qaul.net manual
cd manual
./deploy.sh
cd ..

# build and upload the HTTP-API documentation
cd http-api
./deploy.sh
cd ..

# build and upload the rust documentation
cd ..
cargo doc --no-deps
cp target/doc/settings.html target/doc/index.html
rsync -azzhe "ssh -p 2222" ./target/doc/ admin@docs.qaul.net:/home/admin/api
cd docs

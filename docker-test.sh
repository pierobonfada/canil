#!/bin/bash
docker rm -f canil-test 2>/dev/null
docker rmi -f canil-test 2>/dev/null
docker build --no-cache -t canil-test .
docker run -d --name canil-test -p 8000:8000 -e JWT_SECRET="test1234" canil-test
sleep 5
curl -s -v http://localhost:8000/

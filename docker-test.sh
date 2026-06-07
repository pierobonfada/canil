#!/bin/bash
docker build -t canil-test .
docker rm -f canil-test 2>/dev/null
docker run -d --name canil-test -p 8080:8000 -e JWT_SECRET="test1234" canil-test
sleep 5
curl -s -v http://localhost:8080/

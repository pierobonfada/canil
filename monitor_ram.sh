#!/bin/bash
echo "Monitoring RAM for canil-test..." > ram_usage.log
while true; do
  date >> ram_usage.log
  docker stats --no-stream --format "table {{.MemUsage}}" canil-test | tail -n 1 >> ram_usage.log
  sleep 1
done

#!/usr/bin/env python
import sys
import json
import subprocess
import os

api_url = "https://registry.hub.docker.com/v2/repositories/kylecotton/sdp-backend/tags"
command = "curl -s {}".format(api_url)

data = subprocess.check_output(command, shell=True)
results = json.loads(data)['results']

latest_push_tag = ""
latest_push_date = ""
for r in results:
    date_time = r['tag_last_pushed']
    tag = r['name']

    if date_time > latest_push_date:
        latest_push_date = date_time
        latest_push_tag = tag

print(latest_push_tag)

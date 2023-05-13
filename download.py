# File: download.py
# Author: Chase Ruskin
# Revised: 2023-05-12
# Details:
#   A quick-and-dirty script to download packages
#   from the internet for integration with Orbit.
#
import sys, os
import subprocess

urls = []
# gather list from environment variable specifying the path
if os.getenv("ORBIT_DOWNLOAD_LIST") != None:
    path = os.getenv("ORBIT_DOWNLOAD_LIST")
    f = open(path)
    urls = f.readlines()
    f.close()
else:
    urls = sys.argv[1:]

if len(urls) == 0:
    print('error: Script requires URLs as command-line arguments')
    exit(101)

zips = []
for url in urls:
    url = url.strip()
    if url.endswith('.zip'):
        zips += [url]
    else:
        print("warning: Unsupported URL '"+str(url)+"'")
    pass

print("info: Identifying download destination ...")
# determine the destination to place downloads for future installing
result = subprocess.run(["orbit", "env", "ORBIT_QUEUE"], stdout=subprocess.PIPE)
ORBIT_QUEUE = os.getenv("ORBIT_QUEUE")
print("info: Download directory:", ORBIT_QUEUE)
os.chdir(ORBIT_QUEUE)

print("info: Downloading "+str(len(zips))+" package(s) ...")

dl_count = 0
for z in zips:
    dest = ORBIT_QUEUE+'/'+str(dl_count)+'.zip'
    # download zip files using curl
    subprocess.run(["curl", "-L", z, "-o", dest])
    # unzip contents
    subprocess.run(["unzip", "-qo", dest])
    # remove zip file
    subprocess.run(["rm", dest])
    dl_count += 1
    pass

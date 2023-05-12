import sys
import subprocess

urls = sys.argv[1:]

if len(urls) == 0:
    print('error: Script requires URLs as command-line arguments')
    exit(101)

zips = []
for url in urls:
    if url.endswith('.zip'):
        zips += [url]
    else:
        print("warning: Unsupported URL '"+str(url)+"'")
    pass

print("info: Identifying download destination ...")
# determine the destination to place downloads for future installing
result = subprocess.run(["orbit", "env", "ORBIT_QUEUE"], stdout=subprocess.PIPE)
ORBIT_QUEUE = result.stdout.decode('utf-8').strip()
print("info: Download directory:", ORBIT_QUEUE)

print("info: Downloading "+str(len(zips))+" package(s) ...")
      
dl_count = 0
for z in zips:
    dest = ORBIT_QUEUE+'/'+str(dl_count)+'.zip'
    # download zip files using curl
    subprocess.run(["curl", "-L", z, "-o", dest])
    # unzip contents
    subprocess.run(["unzip", "-qf", dest])
    # remove zip file
    subprocess.run(["rm", dest])
    dl_count += 1
    pass

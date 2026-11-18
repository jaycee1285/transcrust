"""Run a command, write {"wall_secs","peak_rss_kib"} (child peak RSS) to argv[1]."""
import json, resource, subprocess, sys, time
t = time.time()
rc = subprocess.call(sys.argv[2:])
json.dump({"wall_secs": round(time.time() - t, 2),
           "peak_rss_kib": resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss}, open(sys.argv[1], "w"))
sys.exit(rc)

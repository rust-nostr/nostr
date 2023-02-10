import re
import subprocess

regex = r"^.*(nostr-nodejs).*(\d+)\.(\d+)\.(\d+)$"

proc = subprocess.Popen(["git", "log", "-1", "--pretty=%B"], stdout=subprocess.PIPE)
output = proc.stdout.readline().decode('utf-8')

if re.match(regex, output):
    print("Publishing a new release")
    subprocess.call("npm publish --access public", shell=True)
else:
    print("Skip release")
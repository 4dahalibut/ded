import io
import subprocess
with io.open('tst-cmds', 'r') as f:
    test_cmds = [line.strip() for line in f]
for l in test_cmds:
    subprocess.check_call("cat ~/hii | {}".format(l), shell=True)

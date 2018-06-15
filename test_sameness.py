import io
with io.open('tst-cmds', 'r') as f:
    test_cmds = [line.strip() for line in f]
print(test_cmds)

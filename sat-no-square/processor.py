from math import sqrt
cnt = 4000
with open(f'{cnt}.res') as f:a=f.read()

k = {j for i in a.split('\n') for j in i.split(' ')}
slots = [set(), set(), set(), set(), set()]
idxes=[0]*cnt
for i in range(1,cnt+1):
    idx = 0
    if str(i * 3 - 2) in k:
        idx += 4
    if str(i * 3 - 1) in k:
        idx += 2
    if str(i * 3) in k:
        idx += 1
    slots[idx].add(i)
    idxes[i-1] = idx + 1
for i in slots:
    for j in i:
        for k in i:
            if j < k:
                sq = round(sqrt(j+k))
                if sq * sq == j + k:
                    print(f"got {j} + {k} = {sq}^2")
for i in slots:
    print(sorted(i))
print("".join(str(i) for i in idxes))
print("Done.")
    

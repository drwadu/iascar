from random import random
import sys
import random
import os

random.seed(1210993)

p = 'eval/'
for ccg in os.listdir(p):
    if ccg.endswith('.ccg'):
        route = []
        p_ = os.path.join(p, ccg)
        with open(p_, 'r') as f:
            atom_mappings = list(map(lambda m: int(m.split(
                ' ')[-1]), filter(lambda l: l.startswith('c '), f.readlines())))
            for _ in range(3):
                sign = random.randrange(2)
                assumption = random.choice(atom_mappings)
                if sign:
                    route.append(assumption)
                else:
                    route.append(-assumption)
        with open(p_ + '_a', 'w') as g:
            for a in route:
                g.write(f'{a}\n')

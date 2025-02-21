#this script can be used to compare bench results with stockfish to find all mismatches
engine1bench = """h5b5: 666
h5h1: 3959
h5h2: 3916
h5h3: 5038
h5h4: 3466
h5g5: 3377
h5f5: 3388
h5e5: 4070
h5d5: 3485
h5c5: 3652
h5h6: 4325
h5h7: 4576
h5h8: 5072
f4f3: 5229
d6d5: 4106
c7c6: 4853
c7c5: 4017
g4g3: 3810
g4h4: 3160
g4g5: 4236
g4f5: 4437
"""

engine2bench = """f4f3: 5229
d6d5: 4106
c7c6: 4853
c7c5: 4017
h5h1: 3959
h5h2: 3916
h5h3: 5037
h5h4: 3466
h5b5: 666
h5c5: 3652
h5d5: 3485
h5e5: 4070
h5f5: 3388
h5g5: 3377
h5h6: 4325
h5h7: 4576
h5h8: 5072
g4g5: 4236
g4g3: 3810
g4f5: 4437
g4h4: 3160
"""


move_count_map = dict()

engine1moves = engine1bench.splitlines()

for move_count in engine1moves:
    parts = move_count.split(":")

    move = parts[0].strip()
    count = parts[1].strip()

    move_count_map[move] = count;

engine2moves = engine2bench.splitlines()

for move_count in engine2moves:
    parts = move_count.split(":")

    move = parts[0].strip()
    count = parts[1].strip()

    if not move_count_map[move] == count:
        print(move)

# Features
- \>300_000_000 nodes/sec move generation
- UCI compliant
- Magic bitboards
- Fast magic number initialization
- Full legal move generation with orthogonal and diagonal pin bitboards
- Iterative deepening
- NegaMax with alpha-beta pruning
- Quiescence search
- MVV-LVA move ordering
- Search priority for promising moves
- Heatmap based evaluation
- Fixed duration searches
- Time managing searches
- Infinate searches
- Move-flag bitmasks
- Benchmarking tools
- Comprehensive tests

## How to run

```zsh
cargo build --release
```

## How to use

Please see [UCI-documention](http://page.mi.fu-berlin.de/block/uci.htm)

### Supported commands

- uci
- isready
- position startpos
- position startpos moves <moves>
- position fen <fen>
- position fen <fen> moves <moves>
- go <wtime> <btime> <winc> <binc>
- go infinite
- stop
- fen
- state
- board
- legal moves
- bench <depth>
- quit

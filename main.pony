use "collections"
use "random"

actor Main
  new create(env: Env) =>
    try
      let go = Go.create(9)
      let rng = MT

      var color: Stone = Black
      var moves = go.possible_moves(color)
      var num_consecutive_passes: U32 = 0
      while num_consecutive_passes < 2 do
        if moves.size() > 0 then
          num_consecutive_passes = 0
          go.play(moves(rng.int(moves.size())), color)
          env.out.print(go.string())
        else
          num_consecutive_passes = num_consecutive_passes + 1
          if color is Black then
            env.out.print("black passes")
          else
            env.out.print("white passes")
          end
        end
        color = go.opponent(color)
        moves = go.possible_moves(color)
        let s: I32 = 100000
        @usleep[I32](s)
      end
    else
      env.out.print("error")
    end

primitive Empty
primitive Black
primitive White

type Stone is (Empty | Black | White)

class Vertex
  let x: U64
  let y: U64

  new create(x': U64, y': U64) ? =>
    if (x' < 0) or (x' >= 9) or (y' < 0) or (y' >= 9) then
      error
    end
    x = x'
    y = y'

  fun hash(): U64 =>
    x + (10000 * y)

  fun eq(that: box->Vertex): Bool => (x == that.x) and (y == that.y)
  fun ne(that: box->Vertex): Bool => not eq(that)


class Go
  var _board: Array[Array[Stone]]
  var _size: U64
  var _column_labels: String = "ABCDEFGHIKLMNOPORSTU"
  var _vertex_hashes: Map[U64, U64]
  var _past_position_hashes: Set[U64]

  new create(size: U64) =>
    let rng = MT
    _board = Array[Array[Stone]](size)
    _vertex_hashes = Map[U64, U64]()
    _past_position_hashes = Set[U64]()
    for i in Range(0, size) do
      let col = Array[Stone](size)
      for row in Range(0, size) do
        col.push(Empty)
        _vertex_hashes(row + (1000 * (i + 1000))) = rng.u64() // empty
        _vertex_hashes(row + (1000 * (i + 2000))) = rng.u64() // black
        _vertex_hashes(row + (1000 * (i + 3000))) = rng.u64() // white
      end
      _board.push(col)
    end
    _size = size

  fun stoneAt(vertex: Vertex): Stone ? => _board(vertex.y)(vertex.x)

  fun ref play(vertex: Vertex, stone: Stone, force: Bool = false): Bool ? =>
    if (not force) and (not can_play(vertex, stone)) then
      return false
    end
    _board(vertex.y)(vertex.x) = stone
    for n in neighbours(vertex).values() do
      if (stoneAt(n) is opponent(stone)) and dead(n) then
        remove_group(n)
      end
    end

    _past_position_hashes.add(hash())
    true

  fun ref can_play(vertex: Vertex, stone: Stone): Bool ? =>
    // Can't play if the vertex is not empty.
    if not (_board(vertex.y)(vertex.x) is Empty) then
      return false
    end

    // Detect ko.
    let playout = clone()
    playout.play(vertex, stone, true)
    try
      _past_position_hashes(playout.hash())
      // This board position already happened previously - ko!
      @printf[I32]("would be ko!\n".cstring())
      _board(vertex.y)(vertex.x) = Empty
      return false
    else
      _board(vertex.y)(vertex.x) = Empty
    end

    // Can definitely play if the placed stone will have at least one direct
    // freedom,
    for n in neighbours(vertex).values() do
      if stoneAt(n) is Empty then
        return true
      end
    end

    // Don't allow to destroy real eyes.
    var real_eye = true
    let ns = neighbours(vertex)
    if stoneAt(ns(0)) is stone then
      let g = group(ns(0))
      for n in ns.values() do
        let connected = try
          g(n)
          true
        else
          false
        end
        if not connected then
          real_eye = false
        end
      end
    else
      real_eye = false
    end
    if real_eye then
      return false
    end

    // Allow to play if the placed stone will kill at least one group.
    _board(vertex.y)(vertex.x) = stone
    for n in neighbours(vertex).values() do
      if (stoneAt(n) is opponent(stone)) and dead(n) then
        _board(vertex.y)(vertex.x) = Empty
        return true
      end
    end

    // Don't allow to play if the stone would be dead or kill its own group.
    if dead(vertex) then
      _board(vertex.y)(vertex.x) = Empty
      false
    else
      _board(vertex.y)(vertex.x) = Empty
      true
    end

  fun ref possible_moves(stone: Stone): List[Vertex] =>
    let moves = List[Vertex]()
    try
      for row in Range(0, _size) do
        for col in Range(0, _size) do
          let v = Vertex(row, col)
          if (stoneAt(v) is Empty) and can_play(v, stone) then
            moves.push(v)
          end
        end
      end
    end
    moves

  fun dead(vertex: Vertex): Bool ? =>
    for v in group(vertex).values() do
      for n in neighbours(v).values() do
        if stoneAt(n) is Empty then
          return false
        end
      end
    end
    true

  fun ref remove_group(vertex: Vertex) ? =>
    for v in group(vertex).values() do
      _board(v.y)(v.x) = Empty
    end

  fun group(vertex: Vertex): Set[Vertex] ? =>
    let g = Set[Vertex]()
    let candidates = List[Vertex]()
    candidates.push(vertex)
    while candidates.size() > 0 do
      let v = candidates.pop()
      g.add(v)
      for n in neighbours(v).values() do
        if stoneAt(v) is stoneAt(n) then
          try
            g(n)
          else
            g.add(n)
            candidates.push(n)
          end
        end
      end
    end
    g

  fun opponent(s: Stone): Stone =>
    match s
    | Black => White
    | White => Black
    else Empty
    end

  fun neighbours(v: Vertex): List[Vertex] =>
    let ns = List[Vertex]()
    try
      if (v.x > 0) then
        ns.push(Vertex.create(v.x - 1, v.y))
      end
      if (v.y > 0) then
        ns.push(Vertex.create(v.x, v.y - 1))
      end
      if ((v.x + 1) < _size) then
        ns.push(Vertex.create(v.x + 1, v.y))
      end
      if ((v.y + 1) < _size) then
        ns.push(Vertex.create(v.x, v.y + 1))
      end
    end
    ns

  fun hash(): U64 =>
    var h: U64 = 0
    try
      for col in Range(0, _size) do
        for row in Range(0, _size) do
          match stoneAt(Vertex(col, row))
          | Black => h = h + _vertex_hashes(row + (1000 * (col + 2000)))
          | White => h = h + _vertex_hashes(row + (1000 * (col + 3000)))
          else h = h + _vertex_hashes(row + (1000 * (col + 1000)))
          end
        end
      end
    end
    h


  fun string(): String =>
    var str = recover String end

    try
      str.append("   ")
      for col in Range(0, _size) do
        str.append(" ")
        str.append(_column_labels.substring(col.i64(), col.i64()))
      end
      str.append("\n")

      for row in Range(0, _size) do
        str.append((row + 1).string(where width = 2))
        str.append(" \x1b[43m ")
        for col in Range(0, _size) do
          match _board(row)(col)
          | Empty => str.append("\x1b[37m\u00b7 ")
          | Black => str.append("\x1b[30m\u25CF\x1b[37m ")
          | White => str.append("\x1b[37m\u25CF\x1b[37m ")
          end
        end
        str.append("\x1b[49m ")
        str.append((row + 1).string(where width = 2))
        str.append("\n")
      end

      str.append("   ")
      for col in Range(0, _size) do
        str.append(" ")
        str.append(_column_labels.substring(col.i64(), col.i64()))
      end
    end
    str

  fun clone(): Go =>
    let go = Go.create(_size)
    try
      for row in Range(0, _size) do
        for col in Range(0, _size) do
          go._board(row)(col) = _board(row)(col)
        end
      end
    end
    go._vertex_hashes = _vertex_hashes.clone()
    go._past_position_hashes = _past_position_hashes.clone()
    go

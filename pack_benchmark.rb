require 'easy-box-packer'
require 'benchmark'

puts Benchmark.measure {
  EasyBoxPacker.pack(
    container: { dimensions: [200, 300, 400], weight_limit: 5000 },
    items: 5000.times.map {|_i| { dimensions: 3.times.map { Random.random_number(14..22) } , weight: Random.random_number(0.5..1.5) } }
  )
}

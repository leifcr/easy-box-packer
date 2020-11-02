require 'rutie'

module RustBoxPacker
  Rutie.new(:rutie_box_packer).init 'Init_rust_packer', File.expand_path('./target')
end

module EasyBoxPacker
  class << self
    def pack(container:, items:)
      RustPacker.pack(container, items)
    end

    def find_smallest_container_with_limits(items:,limit_dimensions:)
      possible = find_smallest_containers(items: items, max_count: 5)
      l = limit_dimensions.sort
      possible.each do |p|
        return p if (p[0] <= l[0]) && (p[1] <= l[1]) && (p[2] <= l[2])
      end
      # In case there is not match, just use the smallest one...
      possible[0];
    end

    def find_smallest_containers(items:,max_count:)
      possible_containers = []
      invalid_containers  = []

      min_vol = items.map { |h| h[:dimensions].inject(&:*) }.inject(&:+)
      # order items from biggest to smallest
      sorted_items = items.sort_by { |h| h[:dimensions].sort }.reverse

      # base_container = sorted_items.first
      based_container = sorted_items.first

      if sorted_items.size == 1
        return based_container[:dimensions]
      end

      find_possible_container(
        possible_containers: possible_containers,
        invalid_containers: invalid_containers,
        container: based_container[:dimensions],
        items: sorted_items.map {|i| i[:dimensions]},
        item_index: 1,
        min_vol: min_vol)

      count = 1;
      containers = []
      possible_containers.map { |a| a.sort }.sort_by { |a| [a.inject(&:*), a.inject(&:+)] }.each do |c|
        packing = pack(
          container: { dimensions: c },
          items: items)
        if packing[:packings].size == 1 && packing[:errors].size == 0
          count += 1
          containers.push(c)
        end
        break if count >= max_count
      end
      ret_c = []
      containers.each do |c|
        ret_c.push check_container_is_bigger_than_greedy_box({dimensions: c}, items) ? item_greedy_box(items) : c
      end
      ret_c
    end

    def find_smallest_container(items:)
      possible_containers = []
      invalid_containers  = []

      min_vol = items.map { |h| h[:dimensions].inject(&:*) }.inject(&:+)
      # order items from biggest to smallest
      sorted_items = items.sort_by { |h| h[:dimensions].sort }.reverse

      # base_container = sorted_items.first
      based_container = sorted_items.first

      if sorted_items.size == 1
        return based_container[:dimensions]
      end

      find_possible_container(
        possible_containers: possible_containers,
        invalid_containers: invalid_containers,
        container: based_container[:dimensions],
        items: sorted_items.map {|i| i[:dimensions]},
        item_index: 1,
        min_vol: min_vol)

      select_container = possible_containers.map { |a| a.sort }.sort_by { |a| [a.inject(&:*), a.inject(&:+)] }.each do |c|
        packing = pack(
          container: { dimensions: c },
          items: items)
        break c if packing[:packings].size == 1 && packing[:errors].size == 0
      end
      check_container_is_bigger_than_greedy_box({dimensions: select_container}, items) ? item_greedy_box(items) : select_container
    end

    private

    def std(contents)
      n = contents.size
      contents.map!(&:to_f)
      mean = contents.reduce(&:+)/n
      sum_sqr = contents.map {|x| x * x}.reduce(&:+)
      Math.sqrt(((sum_sqr - n * mean * mean)/(n-1)).abs)
    end

    def find_possible_container(possible_containers:,invalid_containers:,container:, items:, item_index:, min_vol:)
      return unless items[item_index]
      c_length, c_width, c_height = container.sort.reverse
      b_length, b_width, b_height = items[item_index].sort.reverse
      c_permutations = [
        [c_width,  c_height, c_length],
        [c_length, c_width,  c_height],
        [c_length, c_height, c_width]
      ]
      b_permutations = [
        [b_width,  b_height, b_length],
        [b_length, b_width,  b_height],
        [b_length, b_height, b_width]
      ]

      tmp_possible_containers = []
      # (1) loops base_container 3 rotations
      c_permutations.each do |c_perm|
        # (2) try to puts items to 3 points, then it will create 3 different possible containers
        b_permutations.each do |b_perm|
          tmp_possible_containers << [  c_perm[0] + b_perm[0],     [c_perm[1], b_perm[1]].max, [c_perm[2], b_perm[2]].max]
          tmp_possible_containers << [ [c_perm[0], b_perm[0]].max,  c_perm[1] + b_perm[1],     [c_perm[2], b_perm[2]].max]
          tmp_possible_containers << [ [c_perm[0], b_perm[0]].max, [c_perm[1], b_perm[1]].max,  c_perm[2]+ b_perm[2]]
        end
      end
      removed_tried_container = tmp_possible_containers.map { |a| a.sort }.uniq - possible_containers - invalid_containers

      return unless removed_tried_container.any?
      # (3) loop all container from smallest spaces to biggest space
      removed_tried_container.sort_by { |a| [a.inject(&:*), a.inject(&:+)] }.each do |cont|
        # (4) next unless l * w * h >= minimum_space
        if cont.inject(&:*) >= min_vol
          possible_containers << cont
        # else
          # puts "invalid: #{cont}"
          # invalid_containers << cont
          # find_possible_container(possible_containers: possible_containers, invalid_containers: invalid_containers, container: cont, items: items, item_index: item_index + 1, min_vol: min_vol)
        end
      end
      # minimum_space = (removed_tried_container).sort_by { |a| [a.inject(&:*), a.inject(&:+)] }.first
      minimum_std = removed_tried_container.sort_by { |a| [std(a), a.inject(&:*), a.inject(&:+)] }.first
      [minimum_std].uniq.compact.each do |cont|
        find_possible_container(possible_containers: possible_containers, invalid_containers: invalid_containers, container: cont, items: items, item_index: item_index + 1, min_vol: min_vol)
      end
    end

    def item_greedy_box(items)
      RustPacker.item_greedy_box(items)
    end

    def check_container_is_bigger_than_greedy_box(container, items)
      RustPacker.check_container_is_bigger_than_greedy_box(container, items)
    end

    def generate_packing_for_greedy_box(items)
      RustPacker.generate_packing_for_greedy_box(items)
    end
  end
end

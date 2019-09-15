extern crate rutie;

use rutie::{AnyObject, Array, Class, Fixnum, Float, Hash, NilClass, Object, Symbol, Boolean};
use std::cmp::Ordering;
use std::ops::Index;

rutie::class!(RustPacker);

type Coordinates = [f64; 3];

fn to_dimension(rb_dimension: &AnyObject) -> f64 {
    match rb_dimension.try_convert_to::<Fixnum>() {
        Ok(i) => i.to_i64() as f64,
        Err(_) => rb_dimension.try_convert_to::<Float>().unwrap().to_f64(),
    }
}

trait RubyArrayConvertible {
    fn from_ruby(array: &Array) -> Self;
    fn to_ruby(&self) -> Array;
}

impl RubyArrayConvertible for Coordinates {
    fn from_ruby(array: &Array) -> Self {
        [
            to_dimension(&array.at(0)),
            to_dimension(&array.at(1)),
            to_dimension(&array.at(2)),
        ]
    }

    fn to_ruby(&self) -> Array {
        let mut array = Array::new();
        array.push(Float::new(self[0]));
        array.push(Float::new(self[1]));
        array.push(Float::new(self[2]));
        array
    }
}

trait RubyFloatConvertible {
    fn to_f(&self) -> f64;
}

impl RubyFloatConvertible for Option<f64> {
    fn to_f(&self) -> f64 {
        match *self {
            Some(f) => f,
            None => 0.0
        }
    }
}

fn cmp_coordinates(a: &Coordinates, b: &Coordinates) -> Ordering {
    if a[0] < b[0] {
        return Ordering::Less;
    }
    if a[0] > b[0] {
        return Ordering::Greater;
    }
    if a[1] < b[1] {
        return Ordering::Less;
    }
    if a[1] > b[1] {
        return Ordering::Greater;
    }
    if a[2] < b[2] {
        return Ordering::Less;
    }
    if a[2] > b[2] {
        return Ordering::Greater;
    }
    Ordering::Equal
}

struct Dimensions {
    dimensions: Coordinates,
    length: f64,
    width: f64,
    height: f64
}

impl Dimensions {
    fn from_array(array: &Coordinates) -> Dimensions {
        let mut sorted = array.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        Dimensions {
            dimensions: *array,
            length: sorted[2],
            width: sorted[1],
            height: sorted[0]
        }
    }

    fn cmp_lwh(&self, other: &Self) -> Ordering {
        cmp_coordinates(&[self.length, self.width, self.height], &[other.length, other.width, other.height])
    }

    fn cmp_hwl(&self, other: &Self) -> Ordering {
        cmp_coordinates(&[self.height, self.width, self.length], &[other.height, other.width, other.length])
    }
}

impl RubyArrayConvertible for Dimensions {
    fn from_ruby(array: &Array) -> Self {
        Self::from_array(&Coordinates::from_ruby(array))
    }

    fn to_ruby(&self) -> Array {
        self.dimensions.to_ruby()
    }
}

impl Index<usize> for Dimensions {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.dimensions[index]
    }
}

struct Item {
    dimensions: Dimensions,
    weight: Option<f64>
}

impl Item {
    fn from_ruby(hash: Hash) -> Item {
        let dimensions = Dimensions::from_ruby(&hash.at(&Symbol::new("dimensions")).try_convert_to::<Array>().unwrap());
        let weight = to_optional_dimension(&hash.at(&Symbol::new("weight")));
        Item { dimensions, weight }
    }
}

struct Container {
    dimensions: Dimensions,
    weight_limit: Option<f64>
}

impl Container {
    fn from_ruby(hash: Hash) -> Container {
        let dimensions = Dimensions::from_ruby(&hash.at(&Symbol::new("dimensions")).try_convert_to::<Array>().unwrap());
        let weight_limit = to_optional_dimension(&hash.at(&Symbol::new("weight_limit")));
        Container { dimensions, weight_limit }
    }
}

fn to_optional_dimension(rb_dimension: &AnyObject) -> Option<f64> {
    match rb_dimension.try_convert_to::<NilClass>() {
        Ok(_) => None,
        Err(_) => Some(to_dimension(&rb_dimension))
    }
}

fn extract_items(rb_array_of_hashes: Array) -> Vec<Item> {
    let mut items : Vec<Item> = Vec::with_capacity(rb_array_of_hashes.length());
    for hash in rb_array_of_hashes {
        items.push(Item::from_ruby(hash.try_convert_to::<Hash>().unwrap()));
    }
    items
}


fn cmp_dimensions_and_position(a: &[Placement; 3], b: &[Placement; 3]) -> Ordering {
    let ordering = a[0].dimensions.cmp_hwl(&b[0].dimensions);
    if ordering != Ordering::Equal {
        return ordering;
    }

    let ordering = a[1].dimensions.cmp_hwl(&b[1].dimensions);
    if ordering != Ordering::Equal {
        return ordering;
    }

    a[2].dimensions.cmp_hwl(&b[2].dimensions)
}

struct RotationAndMargin<'a> {
    rotation: &'a Coordinates,
    sorted_margins: Coordinates,
}

struct Space {
    dimensions: Dimensions,
    position: Coordinates
}

struct Placement {
    dimensions: Dimensions,
    position: Coordinates,
    weight: f64
}

struct Packing {
    placements: Vec<Placement>,
    spaces: Vec<Space>,
    weight: f64
}

impl Placement {
    fn to_ruby(&self) -> Hash {
        let mut hash = Hash::new();
        hash.store(Symbol::new("dimensions"), self.dimensions.to_ruby());
        hash.store(Symbol::new("position"), self.position.to_ruby());
        hash.store(Symbol::new("weight"), Float::new(self.weight));
        hash
    }
}

fn internal_item_greedy_box(items: &[Item]) -> Dimensions {
    let mut max_length : f64 = 0.0;
    let mut max_width : f64 = 0.0;
    let mut total_height : f64 = 0.0;
    for item in items.iter() {
        max_length = max_length.max(item.dimensions.length);
        max_width = max_width.max(item.dimensions.width);
        total_height += item.dimensions.height;
    }
    Dimensions::from_array(&[max_length, max_width, 0.1 * (10.0 * total_height).round()])
}

fn to_rb_placements(placements: &[Placement]) -> Array {
    let mut result = Array::new();
    for placement in placements {
        result.push(placement.to_ruby());
    }
    result
}

rutie::methods!(
    RustPacker,
    _itself,
    fn place(item: Hash, space: Hash) -> AnyObject {
        let item = Item::from_ruby(item.unwrap());
        let space_hash = space.unwrap();
        let space_dimensions = Coordinates::from_ruby(&space_hash.at(&Symbol::new("dimensions")).try_convert_to::<Array>().unwrap());

        let permutations: [Coordinates; 6] = [
            [item.dimensions.width,  item.dimensions.height, item.dimensions.length],
            [item.dimensions.width,  item.dimensions.length, item.dimensions.height],
            [item.dimensions.height, item.dimensions.width,  item.dimensions.length],
            [item.dimensions.height, item.dimensions.length, item.dimensions.width],
            [item.dimensions.length, item.dimensions.width,  item.dimensions.height],
            [item.dimensions.length, item.dimensions.height, item.dimensions.width],
        ];

        let mut possible_rotations_and_margins: Vec<RotationAndMargin> = Vec::with_capacity(6);

        for rotation in permutations.iter() {
            if rotation[0] > space_dimensions[0]
                || rotation[1] > space_dimensions[1]
                || rotation[2] > space_dimensions[2]
            {
                continue;
            }
            let mut sorted_margins = [
                space_dimensions[0] - rotation[0],
                space_dimensions[1] - rotation[1],
                space_dimensions[2] - rotation[2],
            ];
            sorted_margins.sort_by(|a, b| a.partial_cmp(b).unwrap());
            possible_rotations_and_margins.push(RotationAndMargin {
                rotation,
                sorted_margins,
            });
        }

        if possible_rotations_and_margins.len() == 0 {
            return AnyObject::from(NilClass::new().value());
        }

        let mut result = Hash::new();

        possible_rotations_and_margins
            .sort_by(|a, b| cmp_coordinates(&a.sorted_margins, &b.sorted_margins));

        result.store(
            Symbol::new("dimensions"),
            possible_rotations_and_margins[0].rotation.to_ruby(),
        );
        result.store(
            Symbol::new("position"),
            space_hash.at(&Symbol::new("dimensions")),
        );
        match item.weight {
            Some(f) => result.store(Symbol::new("weight"), Float::new(f)),
            None => result.store(Symbol::new("weight"), NilClass::new())
        };

        AnyObject::from(result.value())
    }

    fn break_up_space(space: Hash, placement: Hash) -> Array {
        let space_hash = space.unwrap();
        let placement_hash = placement.unwrap();
        let space_dimensions = Dimensions::from_ruby(&space_hash.at(&Symbol::new("dimensions")).try_convert_to::<Array>().unwrap());
        let space_position = Dimensions::from_ruby(&space_hash.at(&Symbol::new("position")).try_convert_to::<Array>().unwrap());
        let placement_dimensions = Dimensions::from_ruby(&placement_hash.at(&Symbol::new("dimensions")).try_convert_to::<Array>().unwrap());
        let mut possible_spaces: [[Placement; 3]; 6] = [
            [
                Placement {
                    dimensions: Dimensions::from_array(&[
                        space_dimensions[0],
                        space_dimensions[1],
                        space_dimensions[2] - placement_dimensions[2],
                    ]),
                    position: [
                        space_position[0],
                        space_position[1],
                        space_position[2] + placement_dimensions[2],
                    ],
                    weight: 0.0
                },
                Placement {
                    dimensions: Dimensions::from_array(&[
                        space_dimensions[0],
                        space_dimensions[1] - placement_dimensions[1],
                        placement_dimensions[2],
                    ]),
                    position: [
                        space_position[0],
                        space_position[1] + placement_dimensions[1],
                        space_position[2],
                    ],
                    weight: 0.0
                },
                Placement {
                    dimensions: Dimensions::from_array(&[
                        space_dimensions[0] - placement_dimensions[0],
                        placement_dimensions[1],
                        placement_dimensions[2],
                    ]),
                    position: [
                        space_position[0] + placement_dimensions[0],
                        space_position[1],
                        space_position[2],
                    ],
                    weight: 0.0
                },
            ],
            // HEIGHT SPACE => LENGTH => WIDTH
            [
                Placement {
                    dimensions: Dimensions::from_array(&[
                        space_dimensions[0],
                        space_dimensions[1],
                        space_dimensions[2] - placement_dimensions[2],
                    ]),
                    position: [
                        space_position[0],
                        space_position[1],
                        space_position[2] + placement_dimensions[2],
                    ],
                    weight: 0.0
                },
                Placement {
                    dimensions: Dimensions::from_array(&[
                        space_dimensions[0] - placement_dimensions[0],
                        space_dimensions[1],
                        placement_dimensions[2],
                    ]),
                    position: [
                        space_position[0] + placement_dimensions[0],
                        space_position[1],
                        space_position[2],
                    ],
                    weight: 0.0
                },
                Placement {
                    dimensions: Dimensions::from_array(&[
                        placement_dimensions[0],
                        space_dimensions[1] - placement_dimensions[1],
                        placement_dimensions[2],
                    ]),
                    position: [
                        space_position[0],
                        space_position[1] + placement_dimensions[1],
                        space_position[2],
                    ],
                    weight: 0.0
                },
            ],
            // LENGTH SPACE => HEIGHT => WIDTH
            [
                Placement {
                    dimensions: Dimensions::from_array(&[
                        space_dimensions[0] - placement_dimensions[0],
                        space_dimensions[1],
                        space_dimensions[2],
                    ]),
                    position: [
                        space_position[0] + placement_dimensions[0],
                        space_position[1],
                        space_position[2],
                    ],
                    weight: 0.0
                },
                Placement {
                    dimensions: Dimensions::from_array(&[
                        placement_dimensions[0],
                        space_dimensions[1],
                        space_dimensions[2] - placement_dimensions[2],
                    ]),
                    position: [
                        space_position[0],
                        space_position[1],
                        space_position[2] + placement_dimensions[2],
                    ],
                    weight: 0.0
                },
                Placement {
                    dimensions: Dimensions::from_array(&[
                        placement_dimensions[0],
                        space_dimensions[1] - placement_dimensions[1],
                        placement_dimensions[2],
                    ]),
                    position: [
                        space_position[0],
                        space_position[1] + placement_dimensions[1],
                        space_position[2],
                    ],
                    weight: 0.0
                },
            ],
            // LENGTH SPACE => WIDTH  => HEIGHT
            [
                Placement {
                    dimensions: Dimensions::from_array(&[
                        space_dimensions[0] - placement_dimensions[0],
                        space_dimensions[1],
                        space_dimensions[2],
                    ]),
                    position: [
                        space_position[0] + placement_dimensions[0],
                        space_position[1],
                        space_position[2],
                    ],
                    weight: 0.0
                },
                Placement {
                    dimensions: Dimensions::from_array(&[
                        placement_dimensions[0],
                        space_dimensions[1] - placement_dimensions[1],
                        space_dimensions[2],
                    ]),
                    position: [
                        space_position[0],
                        space_position[1] + placement_dimensions[1],
                        space_position[2],
                    ],
                    weight: 0.0
                },
                Placement {
                    dimensions: Dimensions::from_array(&[
                        placement_dimensions[0],
                        placement_dimensions[1],
                        space_dimensions[2] - placement_dimensions[2],
                    ]),
                    position: [
                        space_position[0],
                        space_position[1],
                        space_position[2] + placement_dimensions[2],
                    ],
                    weight: 0.0
                },
            ],
            // WIDTH SPACE  => LENGTH => HEIGHT
            [
                Placement {
                    dimensions: Dimensions::from_array(&[
                        space_dimensions[0],
                        space_dimensions[1] - placement_dimensions[1],
                        space_dimensions[2],
                    ]),
                    position: [
                        space_position[0],
                        space_position[1] + placement_dimensions[1],
                        space_position[2],
                    ],
                    weight: 0.0
                },
                Placement {
                    dimensions: Dimensions::from_array(&[
                        space_dimensions[0] - placement_dimensions[0],
                        placement_dimensions[1],
                        space_dimensions[2],
                    ]),
                    position: [
                        space_position[0] + placement_dimensions[0],
                        space_position[1],
                        space_position[2],
                    ],
                    weight: 0.0
                },
                Placement {
                    dimensions: Dimensions::from_array(&[
                        placement_dimensions[0],
                        placement_dimensions[1],
                        space_dimensions[2] - placement_dimensions[2],
                    ]),
                    position: [
                        space_position[0],
                        space_position[1],
                        space_position[2] + placement_dimensions[2],
                    ],
                    weight: 0.0
                },
            ],
            // WIDTH SPACE  => HEIGHT => LENGTH
            [
                Placement {
                    dimensions: Dimensions::from_array(&[
                        space_dimensions[0],
                        space_dimensions[1] - placement_dimensions[1],
                        space_dimensions[2],
                    ]),
                    position: [
                        space_position[0],
                        space_position[1] + placement_dimensions[1],
                        space_position[2],
                    ],
                    weight: 0.0
                },
                Placement {
                    dimensions: Dimensions::from_array(&[
                        space_dimensions[0],
                        placement_dimensions[1],
                        space_dimensions[2] - placement_dimensions[2],
                    ]),
                    position: [
                        space_position[0],
                        space_position[1],
                        space_position[2] + placement_dimensions[2],
                    ],
                    weight: 0.0
                },
                Placement {
                    dimensions: Dimensions::from_array(&[
                        space_dimensions[0] - placement_dimensions[0],
                        placement_dimensions[1],
                        placement_dimensions[2],
                    ]),
                    position: [
                        space_position[0] + placement_dimensions[0],
                        space_position[1],
                        space_position[2],
                    ],
                    weight: 0.0
                },
            ],
        ];

        // PICK biggest
        possible_spaces.sort_by(|a, b| cmp_dimensions_and_position(&a, &b));
        let biggest = &possible_spaces[5];
        let mut result = Array::new();
        let mut hash = Hash::new();
        hash.store(Symbol::new("dimensions"), biggest[0].dimensions.to_ruby());
        hash.store(Symbol::new("position"), biggest[0].position.to_ruby());
        result.push(hash);
        let mut hash = Hash::new();
        hash.store(Symbol::new("dimensions"), biggest[1].dimensions.to_ruby());
        hash.store(Symbol::new("position"), biggest[1].position.to_ruby());
        result.push(hash);
        let mut hash = Hash::new();
        hash.store(Symbol::new("dimensions"), biggest[2].dimensions.to_ruby());
        hash.store(Symbol::new("position"), biggest[2].position.to_ruby());
        result.push(hash);

        result
    }

    fn item_greedy_box(items: Array) -> Array {
        let items = items.unwrap();
        let items = extract_items(items);
        internal_item_greedy_box(&items).to_ruby()
    }

    fn check_container_is_bigger_than_greedy_box(container: Hash, items: Array) -> Boolean {
        let container = Container::from_ruby(container.unwrap());
        let items = extract_items(items.unwrap());
        let greedy_box = internal_item_greedy_box(&items);
        let mut weight = 0.0;
        for item in items {
            weight += item.weight.to_f();
        }
        let result = container.dimensions.length >= greedy_box[0] &&
            container.dimensions.width >= greedy_box[1]  &&
            container.dimensions.height >= greedy_box[2] &&
            container.weight_limit.to_f() >= weight;

        Boolean::new(result)
    }

    fn generate_packing_for_greedy_box(items: Array) -> Array {
        let items = extract_items(items.unwrap());
        let mut height = 0.0;
        let mut weight = 0.0;
        let mut placements : Vec<Placement> = Vec::with_capacity(items.len());
        for item in items {
            let item_weight = item.weight.to_f();
            weight += item_weight;
            height += item.dimensions.height;
            placements.push( Placement { dimensions: item.dimensions, position: [0.0, 0.0, height], weight: item_weight } );
        }
        let mut result = Array::new();
        let mut return_h = Hash::new();
        return_h.store(Symbol::new("weight"), Float::new(weight));
        return_h.store(Symbol::new("spaces"), Array::new());
        return_h.store(Symbol::new("placements"), to_rb_placements(&placements));
        result.push(return_h);
        result
    }
);

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Init_rust_packer() {
    Class::new("RustPacker", None).define(|itself| {
        itself.def_self("place", place);
        itself.def_self("break_up_space", break_up_space);
        itself.def_self("item_greedy_box", item_greedy_box);
        itself.def_self("check_container_is_bigger_than_greedy_box", check_container_is_bigger_than_greedy_box);
        itself.def_self("generate_packing_for_greedy_box", generate_packing_for_greedy_box);
    });
}

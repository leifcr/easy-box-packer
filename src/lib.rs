extern crate rutie;

use rutie::{AnyObject, Array, Class, Fixnum, Float, Hash, NilClass, Object, Symbol, Boolean, RString};
use std::cmp::Ordering;
use std::ops::Index;
use core::fmt;

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

#[derive(Clone)]
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

    fn volume(&self) -> f64 {
        self.length * self.width * self.height
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

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = write!(f, "{{:dimensions=>[{}, {}, {}]", self.dimensions[0], self.dimensions[1], self.dimensions[2]);
        if result.is_err() {
            return result;
        }
        if let Some(w) = self.weight {
            let result = write!(f, ", weight: {}", w);
            if result.is_err() {
                return result;
            }
        }
        write!(f, "}}")
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

#[derive(Clone)]
struct Space {
    dimensions: Dimensions,
    position: Coordinates
}

impl Space {
    fn to_ruby(&self) -> Hash {
        let mut hash = Hash::new();
        hash.store(Symbol::new("dimensions"), self.dimensions.to_ruby());
        hash.store(Symbol::new("position"), self.position.to_ruby());
        hash
    }
}

#[derive(Clone)]
struct Placement {
    dimensions: Dimensions,
    position: Coordinates,
    weight: Option<f64>
}

impl Placement {
    fn to_ruby(&self) -> Hash {
        let mut hash = Hash::new();
        hash.store(Symbol::new("dimensions"), self.dimensions.to_ruby());
        hash.store(Symbol::new("position"), self.position.to_ruby());
        match self.weight {
            Some(f) => hash.store(Symbol::new("weight"), Float::new(f)),
            None =>    hash.store(Symbol::new("weight"), NilClass::new())
        };
        hash
    }
}

struct Packing {
    placements: Vec<Placement>,
    spaces: Vec<Space>,
    weight: f64
}

impl Packing {
    fn to_ruby(&self) -> Hash {
        let mut hash = Hash::new();
        let mut placement_array = Array::new();
        for placement in &self.placements {
            placement_array.push(placement.to_ruby());
        }
        let mut space_array = Array::new();
        for space in &self.spaces {
            space_array.push(space.to_ruby());
        }
        hash.store(Symbol::new("placements"), placement_array);
        hash.store(Symbol::new("position"), space_array);
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

fn place(item: &Item, space: &Space) -> Option<Placement> {
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
        if rotation[0] > space.dimensions[0]
            || rotation[1] > space.dimensions[1]
            || rotation[2] > space.dimensions[2]
        {
            continue;
        }
        let mut sorted_margins = [
            space.dimensions[0] - rotation[0],
            space.dimensions[1] - rotation[1],
            space.dimensions[2] - rotation[2],
        ];
        sorted_margins.sort_by(|a, b| a.partial_cmp(b).unwrap());
        possible_rotations_and_margins.push(RotationAndMargin {
            rotation,
            sorted_margins,
        });
    }

    if possible_rotations_and_margins.len() == 0 {
        return None;
    }

    possible_rotations_and_margins
        .sort_by(|a, b| cmp_coordinates(&a.sorted_margins, &b.sorted_margins));

    Some(Placement {
        dimensions: Dimensions::from_array(&possible_rotations_and_margins[0].rotation),
        position: space.dimensions.dimensions,
        weight: item.weight
    })
}

fn break_up_space(space: &Space, placement: &Placement) -> [Space; 3] {
    let mut possible_spaces: [[Placement; 3]; 6] = [
        [
            Placement {
                dimensions: Dimensions::from_array(&[
                    space.dimensions[0],
                    space.dimensions[1],
                    space.dimensions[2] - placement.dimensions[2],
                ]),
                position: [
                    space.position[0],
                    space.position[1],
                    space.position[2] + placement.dimensions[2],
                ],
                weight: Some(0.0)
            },
            Placement {
                dimensions: Dimensions::from_array(&[
                    space.dimensions[0],
                    space.dimensions[1] - placement.dimensions[1],
                    placement.dimensions[2],
                ]),
                position: [
                    space.position[0],
                    space.position[1] + placement.dimensions[1],
                    space.position[2],
                ],
                weight: Some(0.0)
            },
            Placement {
                dimensions: Dimensions::from_array(&[
                    space.dimensions[0] - placement.dimensions[0],
                    placement.dimensions[1],
                    placement.dimensions[2],
                ]),
                position: [
                    space.position[0] + placement.dimensions[0],
                    space.position[1],
                    space.position[2],
                ],
                weight: Some(0.0)
            },
        ],
        // HEIGHT SPACE => LENGTH => WIDTH
        [
            Placement {
                dimensions: Dimensions::from_array(&[
                    space.dimensions[0],
                    space.dimensions[1],
                    space.dimensions[2] - placement.dimensions[2],
                ]),
                position: [
                    space.position[0],
                    space.position[1],
                    space.position[2] + placement.dimensions[2],
                ],
                weight: Some(0.0)
            },
            Placement {
                dimensions: Dimensions::from_array(&[
                    space.dimensions[0] - placement.dimensions[0],
                    space.dimensions[1],
                    placement.dimensions[2],
                ]),
                position: [
                    space.position[0] + placement.dimensions[0],
                    space.position[1],
                    space.position[2],
                ],
                weight: Some(0.0)
            },
            Placement {
                dimensions: Dimensions::from_array(&[
                    placement.dimensions[0],
                    space.dimensions[1] - placement.dimensions[1],
                    placement.dimensions[2],
                ]),
                position: [
                    space.position[0],
                    space.position[1] + placement.dimensions[1],
                    space.position[2],
                ],
                weight: Some(0.0)
            },
        ],
        // LENGTH SPACE => HEIGHT => WIDTH
        [
            Placement {
                dimensions: Dimensions::from_array(&[
                    space.dimensions[0] - placement.dimensions[0],
                    space.dimensions[1],
                    space.dimensions[2],
                ]),
                position: [
                    space.position[0] + placement.dimensions[0],
                    space.position[1],
                    space.position[2],
                ],
                weight: Some(0.0)
            },
            Placement {
                dimensions: Dimensions::from_array(&[
                    placement.dimensions[0],
                    space.dimensions[1],
                    space.dimensions[2] - placement.dimensions[2],
                ]),
                position: [
                    space.position[0],
                    space.position[1],
                    space.position[2] + placement.dimensions[2],
                ],
                weight: Some(0.0)
            },
            Placement {
                dimensions: Dimensions::from_array(&[
                    placement.dimensions[0],
                    space.dimensions[1] - placement.dimensions[1],
                    placement.dimensions[2],
                ]),
                position: [
                    space.position[0],
                    space.position[1] + placement.dimensions[1],
                    space.position[2],
                ],
                weight: Some(0.0)
            },
        ],
        // LENGTH SPACE => WIDTH  => HEIGHT
        [
            Placement {
                dimensions: Dimensions::from_array(&[
                    space.dimensions[0] - placement.dimensions[0],
                    space.dimensions[1],
                    space.dimensions[2],
                ]),
                position: [
                    space.position[0] + placement.dimensions[0],
                    space.position[1],
                    space.position[2],
                ],
                weight: Some(0.0)
            },
            Placement {
                dimensions: Dimensions::from_array(&[
                    placement.dimensions[0],
                    space.dimensions[1] - placement.dimensions[1],
                    space.dimensions[2],
                ]),
                position: [
                    space.position[0],
                    space.position[1] + placement.dimensions[1],
                    space.position[2],
                ],
                weight: Some(0.0)
            },
            Placement {
                dimensions: Dimensions::from_array(&[
                    placement.dimensions[0],
                    placement.dimensions[1],
                    space.dimensions[2] - placement.dimensions[2],
                ]),
                position: [
                    space.position[0],
                    space.position[1],
                    space.position[2] + placement.dimensions[2],
                ],
                weight: Some(0.0)
            },
        ],
        // WIDTH SPACE  => LENGTH => HEIGHT
        [
            Placement {
                dimensions: Dimensions::from_array(&[
                    space.dimensions[0],
                    space.dimensions[1] - placement.dimensions[1],
                    space.dimensions[2],
                ]),
                position: [
                    space.position[0],
                    space.position[1] + placement.dimensions[1],
                    space.position[2],
                ],
                weight: Some(0.0)
            },
            Placement {
                dimensions: Dimensions::from_array(&[
                    space.dimensions[0] - placement.dimensions[0],
                    placement.dimensions[1],
                    space.dimensions[2],
                ]),
                position: [
                    space.position[0] + placement.dimensions[0],
                    space.position[1],
                    space.position[2],
                ],
                weight: Some(0.0)
            },
            Placement {
                dimensions: Dimensions::from_array(&[
                    placement.dimensions[0],
                    placement.dimensions[1],
                    space.dimensions[2] - placement.dimensions[2],
                ]),
                position: [
                    space.position[0],
                    space.position[1],
                    space.position[2] + placement.dimensions[2],
                ],
                weight: Some(0.0)
            },
        ],
        // WIDTH SPACE  => HEIGHT => LENGTH
        [
            Placement {
                dimensions: Dimensions::from_array(&[
                    space.dimensions[0],
                    space.dimensions[1] - placement.dimensions[1],
                    space.dimensions[2],
                ]),
                position: [
                    space.position[0],
                    space.position[1] + placement.dimensions[1],
                    space.position[2],
                ],
                weight: Some(0.0)
            },
            Placement {
                dimensions: Dimensions::from_array(&[
                    space.dimensions[0],
                    placement.dimensions[1],
                    space.dimensions[2] - placement.dimensions[2],
                ]),
                position: [
                    space.position[0],
                    space.position[1],
                    space.position[2] + placement.dimensions[2],
                ],
                weight: Some(0.0)
            },
            Placement {
                dimensions: Dimensions::from_array(&[
                    space.dimensions[0] - placement.dimensions[0],
                    placement.dimensions[1],
                    placement.dimensions[2],
                ]),
                position: [
                    space.position[0] + placement.dimensions[0],
                    space.position[1],
                    space.position[2],
                ],
                weight: Some(0.0)
            },
        ],
    ];

    // PICK biggest
    possible_spaces.sort_by(|a, b| cmp_dimensions_and_position(&a, &b));
    let biggest = &possible_spaces[5];
    [
        Space {
            dimensions: biggest[0].dimensions.clone(),
            position: biggest[0].position
        },
        Space {
            dimensions: biggest[1].dimensions.clone(),
            position: biggest[1].position
        },
        Space {
            dimensions: biggest[2].dimensions.clone(),
            position: biggest[2].position
        },
    ]
}

fn internal_check_container_is_bigger_than_greedy_box(container: &Container, items: &[Item]) -> bool {
    let greedy_box = internal_item_greedy_box(&items);
    let mut weight = 0.0;
    for item in items {
        weight += item.weight.to_f();
    }
    container.dimensions.length >= greedy_box[0] &&
        container.dimensions.width >= greedy_box[1]  &&
        container.dimensions.height >= greedy_box[2] &&
        container.weight_limit.to_f() >= weight
}

fn internal_generate_packing_for_greedy_box(items: &[Item]) -> Packing {
    let mut height = 0.0;
    let mut weight = 0.0;
    let mut placements : Vec<Placement> = Vec::with_capacity(items.len());
    for item in items {
        let item_weight = item.weight.to_f();
        weight += item_weight;
        height += item.dimensions.height;
        placements.push( Placement { dimensions: item.dimensions.clone(), position: [0.0, 0.0, height], weight: item.weight } );
    }
    Packing {
        placements,
        spaces: vec![],
        weight
    }
}

rutie::methods!(
    RustPacker,
    _itself,
    fn pack(container: Hash, items: Array) -> Hash {
        let container = Container::from_ruby(container.unwrap());
        let mut items = extract_items(items.unwrap());
        let mut packings: Vec<Packing> = Vec::new();
        let mut errors: Vec<String> = Vec::new();

        // so by length first (biggest) and then sort in descending order
        items.sort_by(|a, b| b.dimensions.cmp_lwh(&a.dimensions));
        for item in &items {
            if item.weight.to_f() > container.weight_limit.to_f() {
                errors.push(format!("Item: {} is too heavy for container", item));
                continue;
            }
            let mut item_has_been_packed = false;
            for packing in &mut packings {
                // If this packings going to be too big with this
                // item as well then skip on to the next packing
                if packing.weight + item.weight.to_f() > container.weight_limit.to_f() {
                    continue;
                }
                // remove volume size = 0 (not possible to pack)
                packing.spaces.retain(|space| space.dimensions.volume() > 0.0);
                // try minimum space first
                packing.spaces.sort_by(|a, b| a.dimensions.cmp_hwl(&b.dimensions));
                for (i, space) in (&packing.spaces).iter().enumerate() {
                    // Try placing the item in this space,
                    // if it doesn't fit skip on the next space
                    let placement = place(&item, space);
                    if let Some(p) = placement {
                        let new_spaces = break_up_space(space, &p);
                        packing.placements.push(p);
                        packing.weight += item.weight.to_f();
                        packing.spaces.remove(i);
                        packing.spaces.extend_from_slice(&new_spaces);
                        item_has_been_packed = true;
                        break;
                    }
                    else {
                        continue;
                    }
                }
                if item_has_been_packed {
                    break;
                }
            }
            if item_has_been_packed {
                continue;
            }

            // Can't fit in any of the spaces for the current packings
            // so lets try a new space the size of the container
            let space = Space {
                dimensions: Dimensions::from_array(&[
                    container.dimensions.length,
                    container.dimensions.width,
                    container.dimensions.height
                ]),
                position: [0.0, 0.0, 0.0]
            };
            let placement = place(&item, &space);
            // If it can't be placed in this space, then it's just
            // too big for the container and we should abandon hope
            match placement {
                None => {
                    errors.push(format!("Item: {} cannot be placed in container", item));
                    continue;
                },
                Some(p) => {
                    // Otherwise lets put the item in a new packing
                    // and break up the remaing free space around it
                    let spaces = break_up_space(&space, &p);
                    packings.push(Packing {
                        placements: [p].to_vec(),
                        weight: item.weight.to_f(),
                        spaces: spaces.to_vec()
                    });
                }
            }
        }

        if packings.len() > 1 && internal_check_container_is_bigger_than_greedy_box(&container, &items) {
            packings.clear();
            errors.clear();
            packings.push(internal_generate_packing_for_greedy_box(&items));
        }
        let mut packing_array = Array::new();
        for packing in packings {
            packing_array.push(packing.to_ruby());
        }
        let mut error_array = Array::new();
        for error in errors {
            error_array.push(RString::new_utf8(&error));
        }

        let mut result = Hash::new();
        result.store(Symbol::new("packings"), packing_array);
        result.store(Symbol::new("errors"), error_array);
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
        let packing = internal_generate_packing_for_greedy_box(&items);
        let mut result = Array::new();
        result.push(packing.to_ruby());
        result
    }
);

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Init_rust_packer() {
    Class::new("RustPacker", None).define(|itself| {
        itself.def_self("pack", pack);
        itself.def_self("item_greedy_box", item_greedy_box);
        itself.def_self("check_container_is_bigger_than_greedy_box", check_container_is_bigger_than_greedy_box);
        itself.def_self("generate_packing_for_greedy_box", generate_packing_for_greedy_box);
    });
}

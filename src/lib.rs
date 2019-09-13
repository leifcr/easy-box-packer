extern crate rutie;

use rutie::{AnyObject, Array, Class, Fixnum, Float, Hash, NilClass, Object, Symbol, VM};
use std::cmp::Ordering;

rutie::class!(RustPacker);

type Dimensions = [f64; 3];

fn to_dimension(rb_dimension: &AnyObject) -> f64 {
    match rb_dimension.try_convert_to::<Fixnum>() {
        Ok(i) => i.to_i64() as f64,
        Err(_) => rb_dimension.try_convert_to::<Float>().unwrap().to_f64(),
    }
}

fn to_dimensions(rb_array: &AnyObject) -> Dimensions {
    let array = rb_array.try_convert_to::<Array>().unwrap();
    [
        to_dimension(&array.at(0)),
        to_dimension(&array.at(1)),
        to_dimension(&array.at(2)),
    ]
}

fn to_array(a: &Dimensions) -> Array {
    let mut rb_array = Array::new();
    rb_array.push(Float::new(a[0]));
    rb_array.push(Float::new(a[1]));
    rb_array.push(Float::new(a[2]));
    rb_array
}

fn cmp_dimensions(a: &Dimensions, b: &Dimensions) -> Ordering {
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

fn cmp_dimensions_and_position(dima: &[DimensionsAndPosition; 3], dimb: &[DimensionsAndPosition; 3]) -> Ordering {
    let mut a = dima[0].dimensions.clone();
    let mut b = dimb[0].dimensions.clone();
    a.sort_by(|a, b| a.partial_cmp(b).unwrap());
    b.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let ordering = cmp_dimensions(&a, &b);
    if ordering != Ordering::Equal {
        return ordering;
    }

    let mut a = dima[1].dimensions.clone();
    let mut b = dimb[1].dimensions.clone();
    a.sort_by(|a, b| a.partial_cmp(b).unwrap());
    b.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let ordering = cmp_dimensions(&a, &b);
    if ordering != Ordering::Equal {
        return ordering;
    }

    let mut a = dima[2].dimensions.clone();
    let mut b = dimb[2].dimensions.clone();
    a.sort_by(|a, b| a.partial_cmp(b).unwrap());
    b.sort_by(|a, b| a.partial_cmp(b).unwrap());
    cmp_dimensions(&a, &b)
}

struct RotationAndMargin<'a> {
    rotation: &'a Dimensions,
    sorted_margins: Dimensions,
}

struct DimensionsAndPosition {
    dimensions: Dimensions,
    position: Dimensions,
}

rutie::methods!(
    RustPacker,
    _itself,
    fn place(item: Hash, space: Hash) -> AnyObject {
        let item_hash = item.unwrap();
        let space_hash = space.unwrap();
        let space_dimensions = to_dimensions(&space_hash.at(&Symbol::new("dimensions")));
        let mut item_dimensions = to_dimensions(&item_hash.at(&Symbol::new("dimensions")));
        item_dimensions.sort_by(|a, b| b.partial_cmp(a).unwrap());

        let permutations: [Dimensions; 6] = [
            [item_dimensions[1], item_dimensions[2], item_dimensions[0]],
            [item_dimensions[1], item_dimensions[0], item_dimensions[2]],
            [item_dimensions[2], item_dimensions[1], item_dimensions[0]],
            [item_dimensions[2], item_dimensions[0], item_dimensions[1]],
            [item_dimensions[0], item_dimensions[1], item_dimensions[2]],
            [item_dimensions[0], item_dimensions[2], item_dimensions[1]],
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
            .sort_by(|a, b| cmp_dimensions(&a.sorted_margins, &b.sorted_margins));

        result.store(
            Symbol::new("dimensions"),
            to_array(&possible_rotations_and_margins[0].rotation),
        );
        result.store(
            Symbol::new("position"),
            space_hash.at(&Symbol::new("dimensions")),
        );
        result.store(Symbol::new("weight"), item_hash.at(&Symbol::new("weight")));

        AnyObject::from(result.value())
    }

    fn break_up_space(space: Hash, placement: Hash) -> Array {
        let space_hash = space.unwrap();
        let placement_hash = placement.unwrap();
        let space_dimensions = to_dimensions(&space_hash.at(&Symbol::new("dimensions")));
        let space_position = to_dimensions(&space_hash.at(&Symbol::new("position")));
        let placement_dimensions = to_dimensions(&placement_hash.at(&Symbol::new("dimensions")));
        let mut possible_spaces: [[DimensionsAndPosition; 3]; 6] = [
            [
                DimensionsAndPosition {
                    dimensions: [
                        space_dimensions[0],
                        space_dimensions[1],
                        space_dimensions[2] - placement_dimensions[2],
                    ],
                    position: [
                        space_position[0],
                        space_position[1],
                        space_position[2] + placement_dimensions[2],
                    ],
                },
                DimensionsAndPosition {
                    dimensions: [
                        space_dimensions[0],
                        space_dimensions[1] - placement_dimensions[1],
                        placement_dimensions[2],
                    ],
                    position: [
                        space_position[0],
                        space_position[1] + placement_dimensions[1],
                        space_position[2],
                    ],
                },
                DimensionsAndPosition {
                    dimensions: [
                        space_dimensions[0] - placement_dimensions[0],
                        placement_dimensions[1],
                        placement_dimensions[2],
                    ],
                    position: [
                        space_position[0] + placement_dimensions[0],
                        space_position[1],
                        space_position[2],
                    ],
                },
            ],
            // HEIGHT SPACE => LENGTH => WIDTH
            [
                DimensionsAndPosition {
                    dimensions: [
                        space_dimensions[0],
                        space_dimensions[1],
                        space_dimensions[2] - placement_dimensions[2],
                    ],
                    position: [
                        space_position[0],
                        space_position[1],
                        space_position[2] + placement_dimensions[2],
                    ],
                },
                DimensionsAndPosition {
                    dimensions: [
                        space_dimensions[0] - placement_dimensions[0],
                        space_dimensions[1],
                        placement_dimensions[2],
                    ],
                    position: [
                        space_position[0] + placement_dimensions[0],
                        space_position[1],
                        space_position[2],
                    ],
                },
                DimensionsAndPosition {
                    dimensions: [
                        placement_dimensions[0],
                        space_dimensions[1] - placement_dimensions[1],
                        placement_dimensions[2],
                    ],
                    position: [
                        space_position[0],
                        space_position[1] + placement_dimensions[1],
                        space_position[2],
                    ],
                },
            ],
            // LENGTH SPACE => HEIGHT => WIDTH
            [
                DimensionsAndPosition {
                    dimensions: [
                        space_dimensions[0] - placement_dimensions[0],
                        space_dimensions[1],
                        space_dimensions[2],
                    ],
                    position: [
                        space_position[0] + placement_dimensions[0],
                        space_position[1],
                        space_position[2],
                    ],
                },
                DimensionsAndPosition {
                    dimensions: [
                        placement_dimensions[0],
                        space_dimensions[1],
                        space_dimensions[2] - placement_dimensions[2],
                    ],
                    position: [
                        space_position[0],
                        space_position[1],
                        space_position[2] + placement_dimensions[2],
                    ],
                },
                DimensionsAndPosition {
                    dimensions: [
                        placement_dimensions[0],
                        space_dimensions[1] - placement_dimensions[1],
                        placement_dimensions[2],
                    ],
                    position: [
                        space_position[0],
                        space_position[1] + placement_dimensions[1],
                        space_position[2],
                    ],
                },
            ],
            // LENGTH SPACE => WIDTH  => HEIGHT
            [
                DimensionsAndPosition {
                    dimensions: [
                        space_dimensions[0] - placement_dimensions[0],
                        space_dimensions[1],
                        space_dimensions[2],
                    ],
                    position: [
                        space_position[0] + placement_dimensions[0],
                        space_position[1],
                        space_position[2],
                    ],
                },
                DimensionsAndPosition {
                    dimensions: [
                        placement_dimensions[0],
                        space_dimensions[1] - placement_dimensions[1],
                        space_dimensions[2],
                    ],
                    position: [
                        space_position[0],
                        space_position[1] + placement_dimensions[1],
                        space_position[2],
                    ],
                },
                DimensionsAndPosition {
                    dimensions: [
                        placement_dimensions[0],
                        placement_dimensions[1],
                        space_dimensions[2] - placement_dimensions[2],
                    ],
                    position: [
                        space_position[0],
                        space_position[1],
                        space_position[2] + placement_dimensions[2],
                    ],
                },
            ],
            // WIDTH SPACE  => LENGTH => HEIGHT
            [
                DimensionsAndPosition {
                    dimensions: [
                        space_dimensions[0],
                        space_dimensions[1] - placement_dimensions[1],
                        space_dimensions[2],
                    ],
                    position: [
                        space_position[0],
                        space_position[1] + placement_dimensions[1],
                        space_position[2],
                    ],
                },
                DimensionsAndPosition {
                    dimensions: [
                        space_dimensions[0] - placement_dimensions[0],
                        placement_dimensions[1],
                        space_dimensions[2],
                    ],
                    position: [
                        space_position[0] + placement_dimensions[0],
                        space_position[1],
                        space_position[2],
                    ],
                },
                DimensionsAndPosition {
                    dimensions: [
                        placement_dimensions[0],
                        placement_dimensions[1],
                        space_dimensions[2] - placement_dimensions[2],
                    ],
                    position: [
                        space_position[0],
                        space_position[1],
                        space_position[2] + placement_dimensions[2],
                    ],
                },
            ],
            // WIDTH SPACE  => HEIGHT => LENGTH
            [
                DimensionsAndPosition {
                    dimensions: [
                        space_dimensions[0],
                        space_dimensions[1] - placement_dimensions[1],
                        space_dimensions[2],
                    ],
                    position: [
                        space_position[0],
                        space_position[1] + placement_dimensions[1],
                        space_position[2],
                    ],
                },
                DimensionsAndPosition {
                    dimensions: [
                        space_dimensions[0],
                        placement_dimensions[1],
                        space_dimensions[2] - placement_dimensions[2],
                    ],
                    position: [
                        space_position[0],
                        space_position[1],
                        space_position[2] + placement_dimensions[2],
                    ],
                },
                DimensionsAndPosition {
                    dimensions: [
                        space_dimensions[0] - placement_dimensions[0],
                        placement_dimensions[1],
                        placement_dimensions[2],
                    ],
                    position: [
                        space_position[0] + placement_dimensions[0],
                        space_position[1],
                        space_position[2],
                    ],
                },
            ],
        ];

        // PICK biggest
        possible_spaces.sort_by(|a, b| cmp_dimensions_and_position(&a, &b));
        let biggest = &possible_spaces[5];
        let mut result = Array::new();
        let mut hash = Hash::new();
        hash.store(Symbol::new("dimensions"), to_array(&biggest[0].dimensions));
        hash.store(Symbol::new("position"), to_array(&biggest[0].position));
        result.push(hash);
        let mut hash = Hash::new();
        hash.store(Symbol::new("dimensions"), to_array(&biggest[1].dimensions));
        hash.store(Symbol::new("position"), to_array(&biggest[1].position));
        result.push(hash);
        let mut hash = Hash::new();
        hash.store(Symbol::new("dimensions"), to_array(&biggest[2].dimensions));
        hash.store(Symbol::new("position"), to_array(&biggest[2].position));
        result.push(hash);

        result
    }

    fn item_greedy_box(items: Array) -> Array {
        let mut max_length : f64 = 0.0;
        let mut max_width : f64 = 0.0;
        let mut total_height : f64 = 0.0;

        let items = items.unwrap();
        for item in items {
            let item = item.try_convert_to::<Hash>().unwrap();
            let mut dimensions = to_dimensions(&item.at(&Symbol::new("dimensions")));
            dimensions.sort_by(|a, b| b.partial_cmp(a).unwrap());
            max_length = max_length.max(dimensions[0]);
            max_width = max_width.max(dimensions[1]);
            total_height += dimensions[2];
        }
        let mut result = Array::new();
        result.push(Float::new(max_length));
        result.push(Float::new(max_width));
        result.push(Float::new(0.1 * (10.0 * total_height).round()));
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
    });
}

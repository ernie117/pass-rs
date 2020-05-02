use std::collections::HashMap;
use std::error::Error;

pub fn build_table_rows(map: HashMap<String, String>) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let mut vec_of_vecs = Vec::new();

    for (key, value) in map {
        let mut new_vec = Vec::new();
        new_vec.push(key);
        new_vec.push(value);
        vec_of_vecs.push(new_vec);
    }

    Ok(vec_of_vecs)
}

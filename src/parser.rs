// written by Benjamin Haeckler

use std::string::*;

const COMMENT_START: char = '#';

pub struct StringParser{
	names: String,
	values: Vec::<String>
}

#[allow(dead_code)]
impl StringParser {
	pub fn new(s: &str) -> Self {

		let mut names = String::new(); // this will store all of the names of the values
		let mut values = Vec::<String>::new(); // this will store the strings of the values
		for n in s.replace('\r', "\n").as_str().split(';') {
			match n.split_once('=') {
				Some((name, value)) => {
					// removes comments and white space
					let cleaned_name = remove_whitespace(remove_enclosed(&name, COMMENT_START, '\n').as_str());

					// if the name is nothing don't add it to the parser
					if cleaned_name.len() <= 0 {
						continue;
					}
					//adds the name
					names.push_str(cleaned_name.as_str());
					names.push_str(" "); // adds a space

					values.push(value.to_string());// adds the value
				}
				None => {continue;}
			}
		}

		Self{
			names:names,
			values:values
		}
	}

	pub fn print_names (&self) {
		println!("{}", self.names);
	}

	pub fn get_string<'a>(&'a self, name: &str) -> Option<&'a String> {
		let mut idx = 0;
		for n in self.names.as_str().split(' ') {
			if n == name {
				if idx < self.values.len() {
					return Some(&self.values[idx]);
				}else {
					return None;
				}
			}
			idx+=1;
		}
		return None;
	}

	pub fn get_as_ints(&self, name: &str) -> Vec<i32> {
		match self.get_string(&name) {
			Some(s) => {
				parse_int_list(s)
			}

			None => {
				Vec::new()
			}
		}
	}

	pub fn get_as_floats(&self, name: &str) -> Vec<f32> {
		match self.get_string(&name) {
			Some(s) => {
				parse_float_list(s)
			}

			None => {
				Vec::new()
			}
		}
	}

	pub fn get_as_strings(&self, name:&str) -> Vec<String> {
		match self.get_string(&name) {
			Some(s) => {
				let mut res = Vec::new();
				for s2 in s.split(',') {
					res.push(remove_whitespace(String::from(s2).as_str()));
				}

				res
			}

			None => {
				Vec::new()
			}
		}
	}

	pub fn get_as_string_literal_or_def(&self, name:&str, def: &str) -> String {
		match self.get_string(&name) {
			Some(v) => {
				String::from(v)
			}

			None => {
				String::from(def)
			}
		}
	}

	pub fn get_int_or_def(&self, name: &str, def: i32) -> i32 {
		match self.get_string(&name) {
			Some(s) => {
				let s2: &str;
				match s.find(',') {
					Some(num) => {
						s2 = &s[0..num];
					}

					None => {
						s2 = &s;
					}
				};

				return parse_int(s2);
			}

			None => {
				return def;
			}
		};
	}

	pub fn get_float_or_def(&self, name: &str, def: f32) -> f32 {
		match self.get_string(&name) {
			Some(s) => {
				let s2: &str;
				match s.find(',') {
					Some(num) => {
						s2 = &s[0..num];
					}

					None => {
						s2 = &s;
					}
				};

				return parse_float(s2);
			}

			None => {
				return def;
			}
		};
	}

	pub fn get_string_or_def(&self, name: &str, def: String) -> String {
		match self.get_string(&name) {
			Some(s) => {
				let s2: &str;
				match s.find(',') {
					Some(num) => {
						s2 = &s[0..num];
					}

					None => {
						s2 = &s;
					}
				};

				return String::from(s2);
			}

			None => {
				return def;
			}
		};
	}
}

#[allow(dead_code)]
pub fn remove_enclosed(s:&str, opening: char, closing: char) -> String {
	let mut ts: &str = s;
	let mut res = String::new();
	loop {
		match ts.split_once(opening) {
			Some((first, second)) => {
				res.push_str(&first);
				match second.split_once(closing) {
					Some((_, second2)) => {
						ts = second2;
					}

					None => {
						break;
					}
				}
			}

			None => {
				res.push_str(ts);
				break;
			}
		}
	}
	res
}


#[allow(dead_code)]
pub fn remove_whitespace(s: &str) -> String {
	let mut res = String::new();
	for c in s.chars() {
		if c != '\n' && c != '\t' && c != ' ' && c != '\r'{
			res.push(c);
		}
	}

	res
}

#[allow(dead_code)]
pub fn parse_int(s: &str) -> i32 {
	let mut neg = 1;
	let mut res = 0;
	for c in s.chars() {
		if '0' <= c && c <= '9'{ // only parse the number if its a digit
			res = res * 10 + (c as i32) -('0' as i32);
		}
		else if c == '-' && res <= 0 {
			neg *= -1;
		}
		else if c == '.' { // end if decimal. This means that it will round down decimal numbers
			return res * neg;
		}
	}

	res * neg
}

#[allow(dead_code)]
pub fn parse_float(s: &str) -> f32 {
	let mut neg:f32 = 1.0;
	let mut res:f32 = 0.0;
	let mut ch = s.chars();
	// before decimal
	for c in &mut ch {
		if '0' <= c && c <= '9' {
			res = res * 10.0 +(c as i32 as f32) -('0' as i32 as f32);
		}
		else if c == '-' && res <= 0.1 {
			neg *= -1.0;
		}
		else if c == '.' {
			break;
		}
	}
	let mut ch_value = 0.1;
	// after decimal
	for c in ch {
		if '0' <= c && c <= '9' {
			res +=((c as i32 as f32) -('0' as i32 as f32)) * ch_value;
			//println!("C {}",(c as i32 as f32) -('0' as i32 as f32));
			ch_value *= 0.1;
		}
	}

	res * neg
}

#[allow(dead_code)]
pub fn parse_int_list(list: &str) -> Vec<i32> {
	let mut list:&str = &list;
	let mut res = Vec::new();
	loop {
		let end;
		match list.find(',') {
			Some(pos) => {
				end = pos;
			},

			None => {
				res.push(parse_int(&list));
				return res;
			}

		}
		res.push(parse_int(&list[0..end]));
		list = &list[end+1..list.len()];
	}
}

#[allow(dead_code)]
pub fn parse_float_list(list: &str) -> Vec<f32> {
	let mut list:&str = &list;
	let mut res = Vec::new();
	loop {
		let end;
		match list.find(',') {
			Some(pos) => {
				end = pos;
			},

			None => {
				res.push(parse_float(&list));
				return res;
			}

		}
		res.push(parse_float(&list[0..end]));
		list = &list[end+1..list.len()];
	}
}

#[allow(dead_code)]
pub fn array_to_string<T>(a:&[T])  -> String where T: ToString{
	let mut s = String::new();
	if a.len() <= 0 {return s;}
	
	s.push_str(a[0].to_string().as_str());

	for i in 1..a.len() {
		s.push_str(", ");
		s.push_str(a[i].to_string().as_str());
	}

	s
}

#[allow(dead_code)]
pub fn serialize(name: &str, val: &str) -> String {
	return format!("{name} = {val};\n");
}




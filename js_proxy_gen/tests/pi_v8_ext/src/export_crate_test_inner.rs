use std::any::Any;
use std::sync::Arc;

use futures::future::{FutureExt, BoxFuture};

use vm_builtin::{buffer::NativeArrayBuffer, external::{NativeObjectAsyncTaskSpawner, NativeObjectAsyncReply, NativeObjectValue, NativeObjectArgs, NativeObject}};

use export_crate::test::inner::*;

/**
 * 发送消息
 */
pub fn static_call_0(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = (*val) as usize;
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let arg_2 = val.clone();
							let result = send(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_1(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = *val;
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let arg_2 = *val;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_2(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = *val;
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let arg_2 = (*val) as usize;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_3(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = *val;
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let arg_2 = val.clone();
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_4(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = (*val) as usize;
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let arg_2 = *val;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_5(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = (*val) as usize;
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let arg_2 = (*val) as usize;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_6(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = (*val) as usize;
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let arg_2 = val.clone();
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_7(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val.clone();
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let arg_2 = *val;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_8(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val.clone();
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let arg_2 = (*val) as usize;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_9(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val.clone();
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let arg_2 = val.clone();
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_10(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = *val;
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let arg_2 = *val;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_11(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = *val;
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let arg_2 = (*val) as usize;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_12(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = *val;
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let arg_2 = val.clone();
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_13(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = (*val) as usize;
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let arg_2 = *val;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_14(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = (*val) as usize;
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let arg_2 = (*val) as usize;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_15(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = (*val) as usize;
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let arg_2 = val.clone();
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_16(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val.clone();
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let arg_2 = *val;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_17(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val.clone();
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let arg_2 = (*val) as usize;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_18(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val.clone();
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let arg_2 = val.clone();
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_19(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = *val;
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let arg_2 = *val;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_20(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = *val;
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let arg_2 = (*val) as usize;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_21(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = *val;
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let arg_2 = val.clone();
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_22(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = (*val) as usize;
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let arg_2 = *val;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_23(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = (*val) as usize;
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let arg_2 = (*val) as usize;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_24(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = (*val) as usize;
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let arg_2 = val.clone();
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_25(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val.clone();
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let arg_2 = *val;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_26(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val.clone();
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let arg_2 = (*val) as usize;
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取数据
 */
pub fn static_call_27(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val.clone();
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let arg_2 = val.clone();
							let result = get_data(arg_0, arg_1, arg_2);
							match result {
								Err(e) => {
									return Some(Err(format!("{:?}", e)));
								},
								Ok(r) => {
									return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 通知
 */
pub fn async_static_call_0(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = &*val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let arg_2 = &*val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_1(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = &*val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let arg_2 = &((*val) as usize);
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_2(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = &*val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let arg_2 = val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_3(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = &*val;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let arg_2 = &*val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_4(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = &*val;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let arg_2 = &((*val) as usize);
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_5(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = &*val;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let arg_2 = val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_6(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = &*val;
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let arg_2 = &*val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_7(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = &*val;
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let arg_2 = &((*val) as usize);
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_8(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = &*val;
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let arg_2 = val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_9(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = &((*val) as usize);
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let arg_2 = &*val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_10(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = &((*val) as usize);
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let arg_2 = &((*val) as usize);
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_11(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = &((*val) as usize);
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let arg_2 = val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_12(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = &((*val) as usize);
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let arg_2 = &*val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_13(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = &((*val) as usize);
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let arg_2 = &((*val) as usize);
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_14(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = &((*val) as usize);
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let arg_2 = val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_15(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = &((*val) as usize);
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let arg_2 = &*val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_16(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = &((*val) as usize);
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let arg_2 = &((*val) as usize);
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_17(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = &((*val) as usize);
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let arg_2 = val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_18(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let arg_2 = &*val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_19(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let arg_2 = &((*val) as usize);
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_20(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let arg_2 = val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_21(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let arg_2 = &*val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_22(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let arg_2 = &((*val) as usize);
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_23(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let arg_2 = val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_24(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val;
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let arg_2 = &*val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_25(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val;
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let arg_2 = &((*val) as usize);
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 通知
 */
pub fn async_static_call_26(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val;
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let arg_2 = val;
								let result = notify(arg_0, arg_1, arg_2).await;
								if let Some(r) = result {
									reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								} else {
									reply(Ok(NativeObjectValue::empty()));
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 释放测试用结构体
 */
pub fn call_0(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
	let result = self_obj.drop();
	return None;
}

/**
 * 复制测试用结构体
 */
pub fn call_1(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let obj_arc = obj.get_ref::<TestStruct<bool, u8>>().unwrap().upgrade().unwrap();
	let self_obj = obj_arc.as_ref();
	let result = self_obj.clone();
	let r = result;
	return Some(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
}

/**
 * 构建测试用结构体
 */
pub fn static_call_28(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = &*val;
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = &*val;
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let arg_2 = *val;
							match &args[3] {
								NativeObjectValue::Bin(val) => {
									let val_ = val.bytes().to_vec();
									let arg_3 = &val_;
									let result = TestStruct::new(arg_0, arg_1, arg_2, arg_3);
									let r = result;
									return Some(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取x的只读引用
 */
pub fn call_2(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let obj_arc = obj.get_ref::<TestStruct<bool, u8>>().unwrap().upgrade().unwrap();
	let self_obj = obj_arc.as_ref();
	let result = self_obj.get_x();
	let r = result;
	match r {
		r if r.is::<bool>() => {
			return Some(Ok(NativeObjectValue::Bool(r)));
		},
		_ => {
			return Some(Err("Invalid return type".to_string()));
		},
	}
}

/**
 * 设置x的只读引用
 */
pub fn call_3(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = &*val;
			let result = self_obj.set_x(arg_0);
			return None;
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 设置指定类型的值
 */
pub fn static_call_29(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			let result = TestStruct::set(arg_0);
			if let Some(r) = result {
				match r {
					r if r.is::<bool>() => {
						return Some(Ok(NativeObjectValue::Bool(r)));
					},
					_ => {
						return Some(Err("Invalid return type".to_string()));
					},
				}
			} else {
				return None;
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 设置指定类型的值
 */
pub fn static_call_30(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			let result = TestStruct::set(arg_0);
			if let Some(r) = result {
				match r {
					r if r.is::<u8>() || r.is::<u16>() || r.is::<u32>() || r.is::<u64>() || r.is::<u128>() || r.is::<usize>() => {
						return Some(Ok(NativeObjectValue::Uint(r as u32)));
					},
					_ => {
						return Some(Err("Invalid return type".to_string()));
					},
				}
			} else {
				return None;
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 设置指定类型的值
 */
pub fn static_call_31(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			let result = TestStruct::set(arg_0);
			if let Some(r) = result {
				match r {
					r if r.is::<String>() => {
						return Some(Ok(NativeObjectValue::Str(r)));
					},
					_ => {
						return Some(Err("Invalid return type".to_string()));
					},
				}
			} else {
				return None;
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_4(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = &*val;
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let mut val_ = *val;
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<bool>() => {
															return Some(Ok(NativeObjectValue::Bool(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_5(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = &*val;
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let mut val_ = ((*val) as usize);
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<u8>() || r.is::<u16>() || r.is::<u32>() || r.is::<u64>() || r.is::<u128>() || r.is::<usize>() => {
															return Some(Ok(NativeObjectValue::Uint(r as u32)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_6(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = &*val;
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let mut val_ = val.clone();
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<String>() => {
															return Some(Ok(NativeObjectValue::Str(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_7(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = &((*val) as usize);
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let mut val_ = *val;
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<bool>() => {
															return Some(Ok(NativeObjectValue::Bool(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_8(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = &((*val) as usize);
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let mut val_ = ((*val) as usize);
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<u8>() || r.is::<u16>() || r.is::<u32>() || r.is::<u64>() || r.is::<u128>() || r.is::<usize>() => {
															return Some(Ok(NativeObjectValue::Uint(r as u32)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_9(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = &((*val) as usize);
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let mut val_ = val.clone();
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<String>() => {
															return Some(Ok(NativeObjectValue::Str(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_10(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val;
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let mut val_ = *val;
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<bool>() => {
															return Some(Ok(NativeObjectValue::Bool(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_11(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val;
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let mut val_ = ((*val) as usize);
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<u8>() || r.is::<u16>() || r.is::<u32>() || r.is::<u64>() || r.is::<u128>() || r.is::<usize>() => {
															return Some(Ok(NativeObjectValue::Uint(r as u32)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_12(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val;
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let mut val_ = val.clone();
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<String>() => {
															return Some(Ok(NativeObjectValue::Str(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 同步指定类型的值
 */
pub fn async_call_0(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_1(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_2(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_3(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_4(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_5(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_6(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_7(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_8(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 释放测试用结构体
 */
pub fn call_13(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
	let result = self_obj.drop();
	return None;
}

/**
 * 复制测试用结构体
 */
pub fn call_14(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let obj_arc = obj.get_ref::<TestStruct<usize, u8>>().unwrap().upgrade().unwrap();
	let self_obj = obj_arc.as_ref();
	let result = self_obj.clone();
	let r = result;
	return Some(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
}

/**
 * 构建测试用结构体
 */
pub fn static_call_32(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = &((*val) as usize);
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = &((*val) as usize);
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let arg_2 = (*val) as usize;
							match &args[3] {
								NativeObjectValue::Bin(val) => {
									let val_ = val.bytes().to_vec();
									let arg_3 = &val_;
									let result = TestStruct::new(arg_0, arg_1, arg_2, arg_3);
									let r = result;
									return Some(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取x的只读引用
 */
pub fn call_15(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let obj_arc = obj.get_ref::<TestStruct<usize, u8>>().unwrap().upgrade().unwrap();
	let self_obj = obj_arc.as_ref();
	let result = self_obj.get_x();
	let r = result;
	match r {
		r if r.is::<u8>() || r.is::<u16>() || r.is::<u32>() || r.is::<u64>() || r.is::<u128>() || r.is::<usize>() => {
			return Some(Ok(NativeObjectValue::Uint(r as u32)));
		},
		_ => {
			return Some(Err("Invalid return type".to_string()));
		},
	}
}

/**
 * 设置x的只读引用
 */
pub fn call_16(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = &((*val) as usize);
			let result = self_obj.set_x(arg_0);
			return None;
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 设置指定类型的值
 */
pub fn static_call_33(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			let result = TestStruct::set(arg_0);
			if let Some(r) = result {
				match r {
					r if r.is::<bool>() => {
						return Some(Ok(NativeObjectValue::Bool(r)));
					},
					_ => {
						return Some(Err("Invalid return type".to_string()));
					},
				}
			} else {
				return None;
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 设置指定类型的值
 */
pub fn static_call_34(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			let result = TestStruct::set(arg_0);
			if let Some(r) = result {
				match r {
					r if r.is::<u8>() || r.is::<u16>() || r.is::<u32>() || r.is::<u64>() || r.is::<u128>() || r.is::<usize>() => {
						return Some(Ok(NativeObjectValue::Uint(r as u32)));
					},
					_ => {
						return Some(Err("Invalid return type".to_string()));
					},
				}
			} else {
				return None;
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 设置指定类型的值
 */
pub fn static_call_35(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			let result = TestStruct::set(arg_0);
			if let Some(r) = result {
				match r {
					r if r.is::<String>() => {
						return Some(Ok(NativeObjectValue::Str(r)));
					},
					_ => {
						return Some(Err("Invalid return type".to_string()));
					},
				}
			} else {
				return None;
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_17(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = &*val;
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let mut val_ = *val;
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<bool>() => {
															return Some(Ok(NativeObjectValue::Bool(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_18(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = &*val;
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let mut val_ = ((*val) as usize);
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<u8>() || r.is::<u16>() || r.is::<u32>() || r.is::<u64>() || r.is::<u128>() || r.is::<usize>() => {
															return Some(Ok(NativeObjectValue::Uint(r as u32)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_19(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = &*val;
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let mut val_ = val.clone();
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<String>() => {
															return Some(Ok(NativeObjectValue::Str(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_20(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = &((*val) as usize);
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let mut val_ = *val;
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<bool>() => {
															return Some(Ok(NativeObjectValue::Bool(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_21(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = &((*val) as usize);
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let mut val_ = ((*val) as usize);
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<u8>() || r.is::<u16>() || r.is::<u32>() || r.is::<u64>() || r.is::<u128>() || r.is::<usize>() => {
															return Some(Ok(NativeObjectValue::Uint(r as u32)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_22(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = &((*val) as usize);
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let mut val_ = val.clone();
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<String>() => {
															return Some(Ok(NativeObjectValue::Str(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_23(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val;
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let mut val_ = *val;
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<bool>() => {
															return Some(Ok(NativeObjectValue::Bool(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_24(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val;
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let mut val_ = ((*val) as usize);
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<u8>() || r.is::<u16>() || r.is::<u32>() || r.is::<u64>() || r.is::<u128>() || r.is::<usize>() => {
															return Some(Ok(NativeObjectValue::Uint(r as u32)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_25(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val;
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let mut val_ = val.clone();
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<String>() => {
															return Some(Ok(NativeObjectValue::Str(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 同步指定类型的值
 */
pub fn async_call_9(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_10(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_11(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_12(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_13(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_14(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_15(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_16(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_17(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 释放测试用结构体
 */
pub fn call_26(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
	let result = self_obj.drop();
	return None;
}

/**
 * 复制测试用结构体
 */
pub fn call_27(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let obj_arc = obj.get_ref::<TestStruct<String, u8>>().unwrap().upgrade().unwrap();
	let self_obj = obj_arc.as_ref();
	let result = self_obj.clone();
	let r = result;
	return Some(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
}

/**
 * 构建测试用结构体
 */
pub fn static_call_36(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val;
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val;
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let arg_2 = val.clone();
							match &args[3] {
								NativeObjectValue::Bin(val) => {
									let val_ = val.bytes().to_vec();
									let arg_3 = &val_;
									let result = TestStruct::new(arg_0, arg_1, arg_2, arg_3);
									let r = result;
									return Some(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 获取x的只读引用
 */
pub fn call_28(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let obj_arc = obj.get_ref::<TestStruct<String, u8>>().unwrap().upgrade().unwrap();
	let self_obj = obj_arc.as_ref();
	let result = self_obj.get_x();
	let r = result;
	match r {
		r if r.is::<String>() => {
			return Some(Ok(NativeObjectValue::Str(r)));
		},
		_ => {
			return Some(Err("Invalid return type".to_string()));
		},
	}
}

/**
 * 设置x的只读引用
 */
pub fn call_29(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val;
			let result = self_obj.set_x(arg_0);
			return None;
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 设置指定类型的值
 */
pub fn static_call_37(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			let result = TestStruct::set(arg_0);
			if let Some(r) = result {
				match r {
					r if r.is::<bool>() => {
						return Some(Ok(NativeObjectValue::Bool(r)));
					},
					_ => {
						return Some(Err("Invalid return type".to_string()));
					},
				}
			} else {
				return None;
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 设置指定类型的值
 */
pub fn static_call_38(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			let result = TestStruct::set(arg_0);
			if let Some(r) = result {
				match r {
					r if r.is::<u8>() || r.is::<u16>() || r.is::<u32>() || r.is::<u64>() || r.is::<u128>() || r.is::<usize>() => {
						return Some(Ok(NativeObjectValue::Uint(r as u32)));
					},
					_ => {
						return Some(Err("Invalid return type".to_string()));
					},
				}
			} else {
				return None;
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 设置指定类型的值
 */
pub fn static_call_39(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			let result = TestStruct::set(arg_0);
			if let Some(r) = result {
				match r {
					r if r.is::<String>() => {
						return Some(Ok(NativeObjectValue::Str(r)));
					},
					_ => {
						return Some(Err("Invalid return type".to_string()));
					},
				}
			} else {
				return None;
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_30(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = &*val;
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let mut val_ = *val;
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<bool>() => {
															return Some(Ok(NativeObjectValue::Bool(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_31(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = &*val;
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let mut val_ = ((*val) as usize);
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<u8>() || r.is::<u16>() || r.is::<u32>() || r.is::<u64>() || r.is::<u128>() || r.is::<usize>() => {
															return Some(Ok(NativeObjectValue::Uint(r as u32)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_32(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Bool(val) => {
			let arg_0 = *val;
			match &args[1] {
				NativeObjectValue::Bool(val) => {
					let arg_1 = &*val;
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let mut val_ = val.clone();
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<String>() => {
															return Some(Ok(NativeObjectValue::Str(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_33(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = &((*val) as usize);
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let mut val_ = *val;
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<bool>() => {
															return Some(Ok(NativeObjectValue::Bool(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_34(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = &((*val) as usize);
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let mut val_ = ((*val) as usize);
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<u8>() || r.is::<u16>() || r.is::<u32>() || r.is::<u64>() || r.is::<u128>() || r.is::<usize>() => {
															return Some(Ok(NativeObjectValue::Uint(r as u32)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_35(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Uint(val) => {
			let arg_0 = (*val) as usize;
			match &args[1] {
				NativeObjectValue::Uint(val) => {
					let arg_1 = &((*val) as usize);
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let mut val_ = val.clone();
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<String>() => {
															return Some(Ok(NativeObjectValue::Str(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_36(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val;
					match &args[2] {
						NativeObjectValue::Bool(val) => {
							let mut val_ = *val;
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<bool>() => {
															return Some(Ok(NativeObjectValue::Bool(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_37(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val;
					match &args[2] {
						NativeObjectValue::Uint(val) => {
							let mut val_ = ((*val) as usize);
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<u8>() || r.is::<u16>() || r.is::<u32>() || r.is::<u64>() || r.is::<u128>() || r.is::<usize>() => {
															return Some(Ok(NativeObjectValue::Uint(r as u32)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 刷新指定类型的值
 */
pub fn call_38(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let args = args.get_args().unwrap();

	let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
	match &args[0] {
		NativeObjectValue::Str(val) => {
			let arg_0 = val.clone();
			match &args[1] {
				NativeObjectValue::Str(val) => {
					let arg_1 = val;
					match &args[2] {
						NativeObjectValue::Str(val) => {
							let mut val_ = val.clone();
							let arg_2 = &mut val_;
							match &args[3] {
								NativeObjectValue::NatObj(val) => {
									let arg_3 = val.get_mut::<Vec<bool>>().unwrap();
									match &args[4] {
										NativeObjectValue::NatObj(val) => {
											let arg_4_arc = val.get_ref::<HashMap<usize, String>>().unwrap().upgrade().unwrap();
											let arg_4 = arg_4_arc.as_ref();
											let result = self_obj.flush(arg_0, arg_1, arg_2, arg_3, arg_4);
											match result {
												Err(e) => {
													return Some(Err(format!("{:?}", e)));
												},
												Ok(r) => {
													match r {
														r if r.is::<String>() => {
															return Some(Ok(NativeObjectValue::Str(r)));
														},
														_ => {
															return Some(Err("Invalid return type".to_string()));
														},
													}
												},
											}
										},
										_ => {
											return Some(Err("Invalid type of 4th parameter".to_string()));
										},
									}
								},
								_ => {
									return Some(Err("Invalid type of 3th parameter".to_string()));
								},
							}
						},
						_ => {
							return Some(Err("Invalid type of 2th parameter".to_string()));
						},
					}
				},
				_ => {
					return Some(Err("Invalid type of 1th parameter".to_string()));
				},
			}
		},
		_ => {
			return Some(Err("Invalid type of 0th parameter".to_string()));
		},
	}
}

/**
 * 同步指定类型的值
 */
pub fn async_call_18(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_19(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_20(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_21(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_22(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_23(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_24(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_25(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_26(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestStruct<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3 = val.get_mut::<Vec<Vec<bool>>>().unwrap();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 释放测试用枚举
 */
pub fn call_39(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let self_obj = obj.get_mut::<TestEnum<bool, u8>>().unwrap();
	let result = self_obj.drop();
	return None;
}

/**
 * 复制测试用枚举
 */
pub fn call_40(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let obj_arc = obj.get_ref::<TestEnum<bool, u8>>().unwrap().upgrade().unwrap();
	let self_obj = obj_arc.as_ref();
	let result = self_obj.clone();
	let r = result;
	return Some(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
}

/**
 * 同步指定类型的值
 */
pub fn async_call_27(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_28(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_29(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_30(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_31(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_32(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_33(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_34(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_35(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<bool, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 释放测试用枚举
 */
pub fn call_41(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let self_obj = obj.get_mut::<TestEnum<usize, u8>>().unwrap();
	let result = self_obj.drop();
	return None;
}

/**
 * 复制测试用枚举
 */
pub fn call_42(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let obj_arc = obj.get_ref::<TestEnum<usize, u8>>().unwrap().upgrade().unwrap();
	let self_obj = obj_arc.as_ref();
	let result = self_obj.clone();
	let r = result;
	return Some(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
}

/**
 * 同步指定类型的值
 */
pub fn async_call_36(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_37(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_38(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_39(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_40(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_41(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_42(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_43(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_44(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<usize, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 释放测试用枚举
 */
pub fn call_43(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let self_obj = obj.get_mut::<TestEnum<String, u8>>().unwrap();
	let result = self_obj.drop();
	return None;
}

/**
 * 复制测试用枚举
 */
pub fn call_44(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {
	let obj_arc = obj.get_ref::<TestEnum<String, u8>>().unwrap().upgrade().unwrap();
	let self_obj = obj_arc.as_ref();
	let result = self_obj.clone();
	let r = result;
	return Some(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
}

/**
 * 同步指定类型的值
 */
pub fn async_call_45(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_46(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_47(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Bool(val) => {
				let arg_0 = *val;
				match &args[1] {
					NativeObjectValue::Bool(val) => {
						let arg_1 = &*val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_48(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_49(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_50(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Uint(val) => {
				let arg_0 = (*val) as usize;
				match &args[1] {
					NativeObjectValue::Uint(val) => {
						let arg_1 = &((*val) as usize);
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_51(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Bool(val) => {
								let mut val_ = *val;
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_52(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Uint(val) => {
								let mut val_ = ((*val) as usize);
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}

/**
 * 同步指定类型的值
 */
pub fn async_call_53(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {
	let task = async move {
		let args = args.get_args().unwrap();

		let self_obj = obj.get_mut::<TestEnum<String, u8>>().unwrap();
		match &args[0] {
			NativeObjectValue::Str(val) => {
				let arg_0 = val.clone();
				match &args[1] {
					NativeObjectValue::Str(val) => {
						let arg_1 = val;
						match &args[2] {
							NativeObjectValue::Str(val) => {
								let mut val_ = val.clone();
								let arg_2 = &mut val_;
								match &args[3] {
									NativeObjectValue::NatObj(val) => {
										let arg_3_arc = val.get_ref::<Vec<Vec<bool>>>().unwrap().upgrade().unwrap();
										let arg_3 = arg_3_arc.as_ref();
										match &args[4] {
											NativeObjectValue::NatObj(val) => {
												let arg_4 = val.get_mut::<HashMap<Vec<usize>, Vec<String>>>().unwrap();
												let result = self_obj.sync(arg_0, arg_1, arg_2, arg_3, arg_4).await;
												match result {
													Err(e) => {
														reply(Err(NativeObjectValue::Str(format!("{:?}", e))));
													},
													Ok(r) => {
														reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));
													},
												}
											},
											_ => {
												reply(Err(NativeObjectValue::Str("Invalid type of 4th parameter".to_string())));
											},
										}
									},
									_ => {
										reply(Err(NativeObjectValue::Str("Invalid type of 3th parameter".to_string())));
									},
								}
							},
							_ => {
								reply(Err(NativeObjectValue::Str("Invalid type of 2th parameter".to_string())));
							},
						}
					},
					_ => {
						reply(Err(NativeObjectValue::Str("Invalid type of 1th parameter".to_string())));
					},
				}
			},
			_ => {
				reply(Err(NativeObjectValue::Str("Invalid type of 0th parameter".to_string())));
			},
		}
	}.boxed();
	spawner(task);
}


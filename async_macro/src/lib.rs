#[allow(unused_extern_crates)]
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned, ToTokens};
use syn::AttributeArgs;
use num_cpus;

/*
* 指定的异步运行时类型
*/
#[derive(Debug, Clone, PartialEq)]
enum AsyncRuntimeType {
    SingleThread(Option<String>),
    MultiThread(Option<String>),
}

impl AsyncRuntimeType {
    //是否是单线程运行时
    fn is_single_thread_runtime(&self) -> bool {
        if let AsyncRuntimeType::SingleThread(_) = &self {
            true
        } else {
            false
        }
    }

    fn from_str(s: &str) -> Result<AsyncRuntimeType, String> {
        if s.is_empty() {
            //未指定运行时类型
            Ok(AsyncRuntimeType::SingleThread(None))
        } else {
            //指定了运行时类型
            let v: Vec<String> = s
                .split(":")
                .map(|s| s.to_string())
                .collect();
            let s = v[0].as_str();
            match s {
                "single_thread" => {
                    if v.len() == 2 {
                        //指定了单线程运行时的任务池
                        Ok(AsyncRuntimeType::SingleThread(Some(v[1].clone())))
                    } else {
                        //未指定单线程运行时的任务池
                        Ok(AsyncRuntimeType::SingleThread(None))
                    }
                },
                "multi_thread" => {
                    if v.len() == 2 {
                        //指定了多线程运行时的任务池
                        Ok(AsyncRuntimeType::MultiThread(Some(v[1].clone())))
                    } else {
                        //未指定多线程运行时的任务池
                        Ok(AsyncRuntimeType::MultiThread(None))
                    }
                },
                _ => Err(format!("No such runtime type `{}`", s)),
            }
        }
    }
}

/*
* 异步运行时配置
*/
#[derive(Debug, Clone)]
struct AsyncRuntimeConfig {
    runtime_type:   AsyncRuntimeType,   //异步运行时类型
    worker_size:    Option<usize>,      //工作者数量
    timer_interval: Option<usize>,      //定时器间隔
}

impl AsyncRuntimeConfig {
    //构建一个默认的异步运行时配置
    fn new() -> Self {
        AsyncRuntimeConfig {
            runtime_type: AsyncRuntimeType::SingleThread(None),
            worker_size: None,
            timer_interval: None,
        }
    }

    //设置运行时类型
    fn set_type(&mut self, runtime_type: syn::Lit, span: Span) -> Result<(), syn::Error> {
        let runtime_str = parse_string(runtime_type, span, "type")?;
        self.runtime_type = AsyncRuntimeType::from_str(&runtime_str)
            .map_err(|err| syn::Error::new(span, err))?;

        Ok(())
    }

    //设置多线程运行时工作者数量
    fn set_worker_size(&mut self, worker_size: syn::Lit, span: Span) -> Result<(), syn::Error> {
        if let AsyncRuntimeType::SingleThread(_) = &self.runtime_type {
            return Err(syn::Error::new(
                span,
                "Set worker size failed, reason: `worker_size` set multi thread runtime",
            ));
        }

        let worker_size = parse_int(worker_size, span, "worker_size")?;
        if worker_size == 0 {
            return Err(syn::Error::new(span, "Set worker size failed, reason: `worker_size` may not be 0"));
        }
        self.worker_size = Some(worker_size);

        Ok(())
    }

    //设置运行时定时器间隔
    fn set_timer_interval(&mut self, timer_interval: syn::Lit, span: Span) -> Result<(), syn::Error> {
        let timer_interval = parse_int(timer_interval, span, "timer_interval")?;
        if timer_interval < 0 {
            return Err(syn::Error::new(span, "Set timer interval failed, reason: `timer_interval` may not be less 0"));
        }
        self.timer_interval = Some(timer_interval);

        Ok(())
    }

    // 构建异步运行时配置
    fn build(mut self) -> Self {
        if !self.runtime_type.is_single_thread_runtime() && self.worker_size.is_none() {
            //当前是多线程运行时，且未设置工作者数量，则设置工作者数量为本机逻辑核数
            self.worker_size = Some(num_cpus::get());
        }

        self
    }
}

//分析整数
fn parse_int(int: syn::Lit, span: Span, field: &str) -> Result<usize, syn::Error> {
    match int {
        syn::Lit::Int(lit) => match lit.base10_parse::<usize>() {
            Ok(value) => Ok(value),
            Err(e) => Err(syn::Error::new(
                span,
                format!("Parse int failed, reason: failed to parse value of `{}` as integer: {}", field, e),
            )),
        },
        _ => Err(syn::Error::new(
            span,
            format!("Parse int failed, reason: failed to parse value of `{}` as integer.", field),
        )),
    }
}

//分析字符串
fn parse_string(int: syn::Lit, span: Span, field: &str) -> Result<String, syn::Error> {
    match int {
        syn::Lit::Str(s) => Ok(s.value()),
        syn::Lit::Verbatim(s) => Ok(s.to_string()),
        _ => Err(syn::Error::new(
            span,
            format!("Parse string failed, reason: failed to parse value of `{}` as string.", field),
        )),
    }
}

///
/// 异步运行时的异步主入口
///
#[proc_macro_attribute]
#[cfg(not(test))]
pub fn pi_async_main(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(item as syn::ItemFn);
    if input.sig.asyncness.is_none() {
        //如果主入口函数不是异步函数，则立即返回错误
        let msg = "Parse async main failed, reason: the main function require async function";
        return syn::Error::new_spanned(&input.sig.ident, msg)
            .to_compile_error()
            .into();
    } else {
        //如果主入口函数是异步函数，则移除异步标记
        input.sig.asyncness = None;
    }
    if input.sig.ident == "main" && !input.sig.inputs.is_empty() {
        let msg = "Parse async main failed, reason: the main function cannot accept arguments";
        return syn::Error::new_spanned(&input.sig.ident, msg)
            .to_compile_error()
            .into();
    }

    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    parse_async_macro(input, args, false).unwrap_or_else(|e| e.to_compile_error().into())
}

///
/// 测试异步函数的入口
///
#[proc_macro_attribute]
pub fn pi_async_test(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(item as syn::ItemFn);
    if input.sig.asyncness.is_none() {
        //如果主入口函数不是异步函数，则立即返回错误
        let msg = "Parse async test failed, reason: the test function require async function";
        return syn::Error::new_spanned(&input.sig.ident, msg)
            .to_compile_error()
            .into();
    } else {
        //如果主入口函数是异步函数，则移除异步标记
        input.sig.asyncness = None;
    }
    if !input.sig.inputs.is_empty() {
        let msg = "Parse async test failed, reason: the test function cannot accept arguments";
        return syn::Error::new_spanned(&input.sig.ident, msg)
            .to_compile_error()
            .into();
    }

    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    parse_async_macro(input, args, true).unwrap_or_else(|e| e.to_compile_error().into())
}

//分析宏参数
fn parse_async_macro(mut input: syn::ItemFn,
                     args: AttributeArgs,
                     is_test: bool) -> Result<TokenStream, syn::Error> {
    let mut config = AsyncRuntimeConfig::new();
    for arg in args {
        match arg {
            syn::NestedMeta::Meta(syn::Meta::NameValue(namevalue)) => {
                let ident = namevalue.path.get_ident();
                if ident.is_none() {
                    let msg = "Parse args failed, reason: must have specified ident";
                    return Err(syn::Error::new_spanned(namevalue, msg));
                }

                match ident.unwrap().to_string().to_lowercase().as_str() {
                    "type" => {
                        config.set_type(
                            namevalue.lit.clone(),
                            syn::spanned::Spanned::span(&namevalue.lit),
                        )?;
                    },
                    "worker_size" => {
                        config.set_worker_size(
                            namevalue.lit.clone(),
                            syn::spanned::Spanned::span(&namevalue.lit),
                        )?;
                    },
                    "timer_interval" => {
                        config.set_timer_interval(
                            namevalue.lit.clone(),
                            syn::spanned::Spanned::span(&namevalue.lit),
                        )?;
                    },
                    name => {
                        let msg = format!(
                            "Parse args failed, reason: unknown attribute {} is specified; expected one of: `type`, `worker_size`, `timer_interval`",
                            name,
                        );
                        return Err(syn::Error::new_spanned(namevalue, msg));
                    }
                }
            },
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "Parse args failed, reason: unknown attribute inside the macro",
                ));
            },
        }
    }

    let config = config.build();

    let (last_stmt_start_span, last_stmt_end_span) = {
        let mut last_stmt = input
            .block
            .stmts
            .last()
            .map(ToTokens::into_token_stream)
            .unwrap_or_default()
            .into_iter();
        let start = last_stmt.next().map_or_else(Span::call_site, |t| t.span());
        let end = last_stmt.last().map_or(start, |t| t.span());
        (start, end)
    };
    //生成创建指定配置的基础代码
    let runtime_type = config.runtime_type.clone();
    let mut rt_builder = match config.runtime_type {
        AsyncRuntimeType::SingleThread(pool_type) => {
            if let Some(pool_type) = pool_type {
                //指定了任务池
                let pool_type_name = syn::Ident::new(&pool_type, Span::call_site());
                quote_spanned! {last_stmt_start_span=>
                    let pool = #pool_type_name::default();
                    let runner = r#async::rt::single_thread::SingleTaskRunner::<(), _>::new(pool);
                }
            } else {
                //未指定任务池
                quote_spanned! {last_stmt_start_span=>
                    let runner = r#async::rt::single_thread::SingleTaskRunner::<(), _>::default();
                }
            }
        },
        AsyncRuntimeType::MultiThread(pool_type) => {
            if let Some(pool_type) = pool_type {
                //指定了任务池
                let pool_type_name = syn::Ident::new(&pool_type, Span::call_site());
                quote_spanned! {last_stmt_start_span=>
                    let pool = #pool_type_name::default();
                    r#async::rt::multi_thread::MultiTaskRuntimeBuilder::<(), _>::new(pool)
                }
            } else {
                //未指定任务池
                quote_spanned! {last_stmt_start_span=>
                    r#async::rt::multi_thread::MultiTaskRuntimeBuilder::<(), _>::default()
                }
            }
        },
    };

    if let Some(worker_size) = config.worker_size {
        //生成设置多线程运行时的初始工作者和工作者数量限制的代码
        rt_builder = quote! { #rt_builder.init_worker_size(#worker_size).set_worker_limit(#worker_size, #worker_size) };

        if let Some(timer_interval) = config.timer_interval {
            //生成设置多线程运行时定时器间隔的代码
            rt_builder = quote! { #rt_builder.set_timer_interval(#timer_interval).build() };
        } else {
            rt_builder = quote! { #rt_builder.build() };
        }
    }

    let body = &input.block;
    let brace_token = input.block.brace_token;
    input.block = match runtime_type {
        AsyncRuntimeType::SingleThread(pool_type) => {
            //生成启动单线程运行时和执行异步主入口的代码
            let timer_interval = if let Some(timer_interval) = config.timer_interval {
                quote! { Some(#timer_interval as u64) }
            } else {
                quote! { None }
            }; //生成设置多线程运行时定时器间隔的代码

            if let Some(pool_type) = pool_type {
                //指定了任务池
                let pool_type_name = syn::Ident::new(&pool_type, Span::call_site());
                syn::parse2(quote_spanned! {last_stmt_end_span =>
                    {
                        #rt_builder
                        let rt = runner.startup().unwrap();
                        let rt_copy = rt.clone();
                        let thread_waker = runner.get_thread_waker().unwrap();
                        r#async::rt::spawn_worker_thread("Default-Main-RT",
                            2 * 1024 * 1024,
                            Arc::new(AtomicBool::new(true)),
                            thread_waker,
                            10,
                            #timer_interval,
                            move || {
                                let now = Instant::now();
                                if let Ok(len) = runner.run() {
                                    (len == 0, now.elapsed())
                                } else {
                                    (true, now.elapsed())
                                }
                            },
                            move || {
                                rt_copy.len()
                            });
                            rt.block_on::<#pool_type_name<()>, _>(async #body);
                    }
                }).unwrap()
            } else {
                //未指定任务池
                syn::parse2(quote_spanned! {last_stmt_end_span =>
                    {
                        #rt_builder
                        let rt = runner.startup().unwrap();
                        let rt_copy = rt.clone();
                        let thread_waker = runner.get_thread_waker().unwrap();
                        r#async::rt::spawn_worker_thread("Default-Main-RT",
                            2 * 1024 * 1024,
                            Arc::new(AtomicBool::new(true)),
                            thread_waker,
                            10,
                            #timer_interval,
                            move || {
                                let now = Instant::now();
                                if let Ok(len) = runner.run() {
                                    (len == 0, now.elapsed())
                                } else {
                                    (true, now.elapsed())
                                }
                            },
                            move || {
                                rt_copy.len()
                            });
                            rt.block_on::<SingleTaskPool<()>, _>(async #body);
                    }
                }).unwrap()
            }
        },
        AsyncRuntimeType::MultiThread(pool_type) => {
            //生成启动多线程运行时和执行异步主入口的代码
            if let Some(pool_type) = pool_type {
                //指定了任务池
                let pool_type_name = syn::Ident::new(&pool_type, Span::call_site());
                syn::parse2(quote_spanned! {last_stmt_end_span=>
                    {
                        #rt_builder.block_on::<#pool_type_name<()>, _>(async #body);
                    }
                }).unwrap()
            } else {
                //未指定任务池
                syn::parse2(quote_spanned! {last_stmt_end_span=>
                    {
                        #rt_builder.block_on::<StealableTaskPool<()>, _>(async #body);
                    }
                }).unwrap()
            }
        },
    };
    input.block.brace_token = brace_token;

    let result = if is_test {
        quote! {
            #[test]
            #input
        }
    } else {
        quote! {
            #input
        }
    };

    Ok(result.into())
}
// The `derive` feature used as a crate using Rutie would. Ruby runs on one
// thread, so this test has no harness: `main` boots the VM and runs every
// case in turn. The generated code needs no `unsafe`, so this test forbids it.
#![forbid(unsafe_code)]

use rutie::{
    derive::{Bytes, RubyClass},
    rubyclass, rubyerror, rubyfunction, rubymethods, rubymodule, AnyException, AnyObject, Array,
    Exception, Integer, Mark, Module, Object, Proc, RString, Symbol, VM,
};
use std::{
    collections::HashMap,
    fmt,
    sync::atomic::{AtomicUsize, Ordering},
};

static DROPPED: AtomicUsize = AtomicUsize::new(0);

fn dropped() -> usize {
    DROPPED.load(Ordering::SeqCst)
}

// Counts its drops, standing in for memory, a lock guard or a file.
struct Resource;

impl Drop for Resource {
    fn drop(&mut self) {
        DROPPED.fetch_add(1, Ordering::SeqCst);
    }
}

#[rubyerror(path = "RutieDerive::Error")]
#[derive(Debug)]
pub enum CounterError {
    #[ruby(path = "RutieDerive::OverflowError")]
    Overflow(i64),
    Negative,
}

impl fmt::Display for CounterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CounterError::Overflow(at) => write!(f, "counter overflow at {}", at),
            CounterError::Negative => write!(f, "negative amount"),
        }
    }
}

#[rubyclass(path = "RutieDerive::Stats::Counter")]
pub struct Counter {
    #[ruby(get, set)]
    count: i64,
    #[ruby(get, name = "label")]
    name: String,
    _resource: Resource,
}

#[rubymethods]
impl Counter {
    // `initialize(start = nil, name: "counter")`
    pub fn new(start: Option<i64>, #[ruby(keyword)] name: Option<String>) -> Self {
        Counter {
            count: start.unwrap_or(0),
            name: name.unwrap_or_else(|| "counter".to_string()),
            _resource: Resource,
        }
    }

    pub fn add(&mut self, amount: i64) -> Result<i64, CounterError> {
        let _held = Resource;

        if amount < 0 {
            return Err(CounterError::Negative);
        }

        self.count = self
            .count
            .checked_add(amount)
            .ok_or(CounterError::Overflow(self.count))?;

        Ok(self.count)
    }

    #[ruby(name = "to_s")]
    pub fn describe(&self) -> String {
        format!("#<Counter {} {}>", self.name, self.count)
    }

    #[ruby(name = "zero?")]
    pub fn is_zero(&self) -> bool {
        self.count == 0
    }

    // Another `#[rubyclass]` by reference and mutably.
    pub fn merge(&mut self, other: &Counter) -> i64 {
        self.count += other.count;
        self.count
    }

    pub fn give(&mut self, other: &mut Counter, amount: i64) -> (i64, i64) {
        self.count -= amount;
        other.count += amount;
        (self.count, other.count)
    }

    // Yields while borrowed: the block may reach this object again.
    pub fn each_step(
        &mut self,
        steps: u8,
        #[ruby(block)] block: Proc,
    ) -> Result<i64, AnyException> {
        for _ in 0..steps {
            self.count += 1;
            block.call(&[Integer::new(self.count).to_any_object()]);
        }

        Ok(self.count)
    }

    pub fn peek(&self, #[ruby(block)] block: Option<Proc>) -> i64 {
        if let Some(block) = block {
            block.call(&[]);
        }

        self.count
    }

    pub fn explode(&self) -> i64 {
        let _held = Resource;

        panic!("exploded at {}", self.count)
    }

    pub fn origin() -> Self {
        Counter::new(Some(0), Some("origin".to_string()))
    }

    #[ruby(private)]
    pub fn secret(&self) -> i64 {
        42
    }

    #[ruby(protected)]
    pub fn shared_secret(&self) -> i64 {
        43
    }

    pub fn reveal(&self, other: &AnyObject) -> AnyObject {
        other.protect_send("shared_secret", &[]).unwrap()
    }

    #[ruby(skip)]
    pub fn helper(&self, _not_for_ruby: &mut Vec<u8>) {}
}

// A second block of methods for the same class.
#[rubymethods(part = 1)]
impl Counter {
    pub fn double(&mut self) -> i64 {
        self.count *= 2;
        self.count
    }
}

// Conversions, keywords, splat and the GVL.
#[rubyclass(path = "RutieDerive::Kit")]
#[derive(Clone)]
pub struct Kit;

#[rubymethods]
impl Kit {
    pub fn new() -> Self {
        Kit
    }

    pub fn numbers(&self, small: u8, big: i128, ratio: f64, flag: bool, letter: char) -> String {
        format!("{} {} {} {} {}", small, big, ratio, flag, letter)
    }

    pub fn text(&self, borrowed: &str, owned: String, bytes: &[u8]) -> (usize, String, usize) {
        (borrowed.len(), owned.to_uppercase(), bytes.len())
    }

    pub fn collections(&self, list: Vec<i64>, map: HashMap<String, i64>) -> (i64, Vec<String>) {
        let mut keys: Vec<String> = map.keys().cloned().collect();
        keys.sort();

        (list.iter().sum::<i64>() + map.values().sum::<i64>(), keys)
    }

    pub fn options(
        &self,
        first: i64,
        second: Option<i64>,
        #[ruby(splat)] rest: Vec<i64>,
        #[ruby(keyword)] scale: i64,
        #[ruby(keyword)] offset: Option<i64>,
    ) -> i64 {
        (first + second.unwrap_or(0) + rest.iter().sum::<i64>()) * scale + offset.unwrap_or(0)
    }

    pub fn binary(&self) -> Bytes {
        Bytes(vec![0, 255, 1])
    }

    pub fn nothing(&self) -> Option<String> {
        None
    }

    pub fn failing(&self) -> Result<(), std::io::Error> {
        Err(std::io::Error::from_raw_os_error(2))
    }

    #[ruby(without_gvl)]
    pub fn sleep_without_gvl(&self, milliseconds: u64) -> u64 {
        std::thread::sleep(std::time::Duration::from_millis(milliseconds));
        milliseconds
    }

    // `_dump` and `_load` give Marshal support.
    #[ruby(name = "_dump")]
    pub fn dump(&self, _level: i64) -> String {
        "kit".to_string()
    }

    #[ruby(name = "_load")]
    pub fn load(data: String) -> Result<Self, String> {
        if data == "kit" {
            Ok(Kit)
        } else {
            Err(format!("bad dump: {}", data))
        }
    }
}

// Ruby objects held in fields, marked without a `mark` function.
#[derive(Mark)]
pub struct Pair {
    left: RString,
    right: Option<RString>,
    #[ruby(no_mark)]
    _count: usize,
}

#[rubyclass(path = "RutieDerive::Store")]
pub struct Store {
    list: Vec<AnyObject>,
    map: HashMap<String, RString>,
    pair: Option<Pair>,
    array: Array,
}

fn copy<T: Object>(object: &T) -> T {
    T::from(object.value())
}

impl Clone for Pair {
    fn clone(&self) -> Self {
        Pair {
            left: copy(&self.left),
            right: self.right.as_ref().map(copy),
            _count: self._count,
        }
    }
}

// The copy shares the Ruby objects, as Ruby's `dup` does.
impl Clone for Store {
    fn clone(&self) -> Self {
        Store {
            list: self.list.iter().map(copy).collect(),
            map: self
                .map
                .iter()
                .map(|(key, value)| (key.clone(), copy(value)))
                .collect(),
            pair: self.pair.clone(),
            array: copy(&self.array),
        }
    }
}

#[rubymethods]
impl Store {
    pub fn new() -> Self {
        Store {
            list: Vec::new(),
            map: HashMap::new(),
            pair: None,
            array: Array::new(),
        }
    }

    // Objects made here are referenced only by the struct.
    pub fn fill(&mut self, count: usize) -> usize {
        for index in 0..count {
            let text = format!("item {}", index);

            self.list.push(RString::new_utf8(&text).to_any_object());
            self.map.insert(text.clone(), RString::new_utf8(&text));
            self.array.push(RString::new_utf8(&text));
        }

        self.pair = Some(Pair {
            left: RString::new_utf8("left"),
            right: Some(RString::new_utf8("right")),
            _count: count,
        });

        count
    }

    pub fn check(&self) -> Result<usize, String> {
        for index in 0..self.list.len() {
            let text = format!("item {}", index);
            let listed = self.list[index]
                .try_convert_to::<RString>()
                .map_err(|_| "list lost a string")?;
            let mapped = self.map.get(&text).ok_or("map lost a key")?;
            let arrayed = self
                .array
                .at(index as i64)
                .try_convert_to::<RString>()
                .map_err(|_| "array lost a string")?;

            if listed.to_str() != text || mapped.to_str() != text || arrayed.to_str() != text {
                return Err(format!("item {} changed", index));
            }
        }

        let pair = self.pair.as_ref().ok_or("no pair")?;

        if pair.left.to_str() != "left"
            || pair.right.as_ref().map(|right| right.to_str()) != Some("right")
        {
            return Err("pair changed".to_string());
        }

        Ok(self.list.len())
    }

    // Runs the GC (and compaction) while `self` is borrowed.
    pub fn check_during(&self, #[ruby(block)] block: Proc) -> Result<usize, String> {
        block.call(&[]);
        self.check()
    }
}

// Inheritance between Rust structs.
#[rubyclass(path = "RutieDerive::Shape")]
#[derive(Clone)]
pub struct Shape {
    #[ruby(get)]
    sides: i64,
}

#[rubymethods]
impl Shape {
    pub fn new(sides: i64) -> Self {
        Shape { sides }
    }

    pub fn describe(&self) -> String {
        format!("{} sides", self.sides)
    }

    pub fn same_sides(&self, other: &Shape) -> bool {
        self.sides == other.sides
    }
}

#[rubyclass(path = "RutieDerive::Square", extends = Shape)]
#[derive(Clone)]
pub struct Square {
    #[ruby(base)]
    shape: Shape,
    #[ruby(get)]
    length: f64,
}

#[rubymethods]
impl Square {
    pub fn new(length: f64) -> Self {
        Square {
            shape: Shape { sides: 4 },
            length,
        }
    }

    pub fn area(&self) -> f64 {
        self.length * self.length
    }

    pub fn only_squares(&self, other: &Square) -> f64 {
        other.length
    }
}

// Objects only from Rust.
#[rubyclass(path = "RutieDerive::Token", allocator = false)]
pub struct Token {
    #[ruby(get)]
    value: String,
}

#[rubymodule(path = "RutieDerive::Greeter")]
mod greeter {
    use rutie::{rubyclass, rubymethods};

    #[ruby(constant)]
    const VERSION: &str = "1.2.3";

    #[ruby(constant, name = "LIMIT")]
    const MAXIMUM: i64 = 10;

    pub fn greet(who: &str, #[ruby(keyword)] punctuation: Option<String>) -> String {
        format!(
            "Hello, {}{}",
            who,
            punctuation.unwrap_or_else(|| "!".to_string())
        )
    }

    #[ruby(module_function)]
    pub fn shout(text: &str) -> String {
        text.to_uppercase()
    }

    pub fn token(value: String) -> super::Token {
        super::Token { value }
    }

    #[rubyclass(path = "RutieDerive::Greeter::Card")]
    pub struct Card {
        #[ruby(get)]
        pub text: String,
    }

    #[rubymethods]
    impl Card {
        pub fn new(text: String) -> Self {
            Card { text }
        }
    }
}

#[rubyfunction]
fn standalone(who: &RString, punctuation: Option<&RString>) -> String {
    format!(
        "Hello, {}{}",
        who.to_str(),
        punctuation.map_or("!".to_string(), |p| p.to_str().to_string())
    )
}

fn eval(code: &str) -> AnyObject {
    match VM::eval(code) {
        Ok(value) => value,
        Err(error) => panic!(
            "{} raised {}: {}",
            code,
            class_name(&error),
            error.message()
        ),
    }
}

fn class_name(error: &AnyException) -> String {
    error
        .class()
        .protect_send("name", &[])
        .and_then(|name| name.try_convert_to::<RString>())
        .map(|name| name.to_string())
        .unwrap_or_default()
}

fn eval_err(code: &str) -> AnyException {
    match VM::eval(code) {
        Ok(value) => panic!("{} returned {:?} instead of raising", code, value),
        Err(error) => error,
    }
}

fn int(code: &str) -> i64 {
    eval(code).try_convert_to::<Integer>().unwrap().to_i64()
}

fn string(code: &str) -> String {
    eval(code).try_convert_to::<RString>().unwrap().to_string()
}

fn raises(code: &str, class: &str, message: &str) {
    let error = eval_err(code);

    assert_eq!(
        class_name(&error),
        class,
        "{} raised {}: {}",
        code,
        class_name(&error),
        error.message()
    );
    assert!(
        error.message().contains(message),
        "{} raised {:?}, expected it to contain {:?}",
        code,
        error.message(),
        message
    );
}

fn main() {
    VM::init();
    VM::init_loadpath();

    let counter_class = Counter::define_ruby();
    Kit::define_ruby();
    Store::define_ruby();
    Square::define_ruby();
    Token::define_ruby();
    CounterError::define_ruby();
    greeter::define_ruby();
    Module::new("RutieDerive").def_self("standalone", standalone::ruby);
    eval("C = RutieDerive::Stats::Counter");

    let mut cases = 0;
    let mut case = |name: &str, test: &dyn Fn()| {
        test();
        cases += 1;
        println!("ok - {}", name);
    };

    case("the class is defined under its modules and cached", &|| {
        assert_eq!(Counter::class().unwrap().value(), counter_class.value());
        assert_eq!(string("C.name"), "RutieDerive::Stats::Counter");
    });

    case("new is initialize; Ruby subclasses call super", &|| {
        assert_eq!(int("C.new.count"), 0);
        assert_eq!(string("C.new(5, name: 'hits').to_s"), "#<Counter hits 5>");
        eval("class RutieSubCounter < C; def initialize(n); super(n * 2, name: 'sub'); end; def triple; count * 3; end; end");
        assert_eq!(int("RutieSubCounter.new(7).triple"), 42);
        assert_eq!(
            string("RutieSubCounter.new(1).class.name"),
            "RutieSubCounter"
        );
        assert_eq!(
            string("RutieSubCounter.origin.class.name"),
            "RutieSubCounter"
        );
        assert!(eval("C.private_instance_methods.include?(:initialize)").is_true());
    });

    case(
        "instance methods, renames, visibility, skip and parts",
        &|| {
            assert_eq!(int("c = C.new(40); c.add(2); c.count"), 42);
            assert!(eval("C.new.zero?").is_true());
            assert_eq!(int("C.new(21).double"), 42);
            raises("C.new.secret", "NoMethodError", "private method");
            raises("C.new.shared_secret", "NoMethodError", "protected method");
            assert_eq!(int("C.new.reveal(C.new)"), 43);
            assert!(eval("C.new.respond_to?(:helper)").is_false());
        },
    );

    case("accessors", &|| {
        assert_eq!(int("c = C.new(1); c.count = 9; c.count"), 9);
        assert_eq!(string("C.new(name: 'x').label"), "x");
        raises(
            "C.new.count = 'nine'",
            "TypeError",
            "no implicit conversion of String into Integer",
        );
        assert!(eval("C.new.respond_to?(:label=)").is_false());
    });

    case(
        "fixed arity is Ruby's; variadic methods check it themselves",
        &|| {
            assert_eq!(int("C.instance_method(:add).arity"), 1);
            assert_eq!(int("C.instance_method(:give).arity"), 2);
            assert_eq!(int("C.instance_method(:initialize).arity"), -1);
            raises(
                "C.new.add",
                "ArgumentError",
                "wrong number of arguments (given 0, expected 1)",
            );
            raises(
                "C.new(1, 2)",
                "ArgumentError",
                "wrong number of arguments (given 2, expected 0..1)",
            );
            raises(
                "RutieDerive::Kit.new.options",
                "ArgumentError",
                "(given 0, expected 1+)",
            );
        },
    );

    case(
        "Err is raised after the method's values are dropped",
        &|| {
            let before = dropped();
            raises(
                &format!("C.new({}).add(1)", i64::MAX),
                "RutieDerive::OverflowError",
                "counter overflow at",
            );
            // `_held` in `add`; the counter itself is freed later by the GC.
            assert_eq!(dropped(), before + 1);
            raises("C.new.add(-1)", "RutieDerive::Error", "negative amount");
            assert!(
                eval("RutieDerive::OverflowError.ancestors.include?(RutieDerive::Error)").is_true()
            );
            assert_eq!(
                int("100.times.count { C.new(1).add(-1) rescue RutieDerive::Error }"),
                100
            );
            raises(
                "RutieDerive::Kit.new.failing",
                "Errno::ENOENT",
                "No such file",
            );
        },
    );

    case("wrong types and ranges", &|| {
        raises(
            "C.new.add('2')",
            "TypeError",
            "no implicit conversion of String into Integer",
        );
        raises("C.new('2')", "TypeError", "String into Integer");
        raises(
            "RutieDerive::Kit.new.numbers(300, 1, 1.0, true, 'x')",
            "RangeError",
            "out of range of u8",
        );
        raises(
            "RutieDerive::Kit.new.numbers(1, 1, 1.0, nil, 'x')",
            "TypeError",
            "nil into true or false",
        );
        raises(
            "RutieDerive::Kit.new.numbers(1, 1, 1.0, true, 'xy')",
            "ArgumentError",
            "one character",
        );
        raises(
            "C.new.merge(RutieDerive::Kit.new)",
            "TypeError",
            "wrong argument type RutieDerive::Kit (expected RutieDerive::Stats::Counter)",
        );
        raises(
            "RutieDerive::Kit.new.text(\"\\xff\", 'a', 'b')",
            "ArgumentError",
            "invalid byte sequence",
        );
    });

    case("Rust types in and out", &|| {
        assert_eq!(
            string("RutieDerive::Kit.new.numbers(255, 2**100, 2, false, 'é')"),
            "255 1267650600228229401496703205376 2 false é"
        );
        assert_eq!(
            string("RutieDerive::Kit.new.text('héllo', 'abc', \"\\x00\\x01\").inspect"),
            "[6, \"ABC\", 2]"
        );
        assert_eq!(
            string("RutieDerive::Kit.new.collections([1, 2], { 'a' => 3, 'b' => 4 }).inspect"),
            "[10, [\"a\", \"b\"]]"
        );
        assert_eq!(
            string("RutieDerive::Kit.new.binary.encoding.name"),
            "ASCII-8BIT"
        );
        assert_eq!(int("RutieDerive::Kit.new.binary.bytesize"), 3);
        assert!(eval("RutieDerive::Kit.new.nothing").is_nil());
        raises(
            "RutieDerive::Kit.new.collections([1, 'x'], {})",
            "TypeError",
            "String into Integer",
        );
    });

    case("optional, splat and keyword parameters", &|| {
        assert_eq!(int("RutieDerive::Kit.new.options(1, scale: 2)"), 2);
        assert_eq!(
            int("RutieDerive::Kit.new.options(1, 2, 3, 4, scale: 10, offset: 5)"),
            105
        );
        assert_eq!(
            int("RutieDerive::Kit.new.options(1, nil, scale: 1, offset: nil)"),
            1
        );
        raises(
            "RutieDerive::Kit.new.options(1)",
            "ArgumentError",
            "missing keyword: :scale",
        );
        raises(
            "RutieDerive::Kit.new.options(1, scale: 1, size: 2, color: 3)",
            "ArgumentError",
            "unknown keywords: :size, :color",
        );
        raises(
            "RutieDerive::Kit.new.options(1, scale: 'x')",
            "TypeError",
            "String into Integer",
        );
        // A Hash passed positionally is not keywords.
        raises(
            "RutieDerive::Kit.new.options(1, { scale: 2 })",
            "TypeError",
            "Hash into Integer",
        );
    });

    case("blocks", &|| {
        assert_eq!(int("sum = 0; C.new.each_step(3) { |n| sum += n }; sum"), 6);
        raises("C.new.each_step(1)", "LocalJumpError", "no block given");
        assert_eq!(int("C.new(4).peek"), 4);
        assert_eq!(int("hit = 0; C.new(4).peek { hit += 1 }; hit"), 1);
    });

    case("a re-entrant borrow raises instead of aliasing", &|| {
        // `each_step` holds `&mut self` while the block runs.
        raises(
            "c = C.new; c.each_step(1) { c.count }",
            "RuntimeError",
            "already borrowed mutably",
        );
        raises(
            "c = C.new; c.each_step(1) { c.add(1) }",
            "RuntimeError",
            "already borrowed mutably",
        );
        raises(
            "c = C.new; c.peek { c.double }",
            "RuntimeError",
            "already borrowed (can't change it",
        );
        // Shared borrows nest.
        assert_eq!(int("c = C.new(3); c.peek { c.peek { c.count } }"), 3);
        // The same object as receiver and `&mut` argument, or `&` argument.
        raises(
            "c = C.new; c.give(c, 1)",
            "RuntimeError",
            "already borrowed",
        );
        raises("c = C.new; c.merge(c)", "RuntimeError", "already borrowed");
        assert_eq!(
            string("a = C.new(5); b = C.new(1); a.give(b, 2).inspect"),
            "[3, 3]"
        );
    });

    case(
        "a Ruby exception through a method releases its borrows",
        &|| {
            let before = dropped();
            raises("$rutie_c = C.new(1); $rutie_c.each_step(5) { |n| raise ArgumentError, 'stop' if n == 3 }", "ArgumentError", "stop");
            assert_eq!(int("$rutie_c.count"), 3);
            assert_eq!(int("$rutie_c.add(1)"), 4);
            // `each_step` adds 1 before yielding, so the count is 5 after `throw`.
            assert_eq!(
                int("catch(:done) { $rutie_c.each_step(9) { throw :done, 99 } }"),
                99
            );
            assert_eq!(
                int("[1].each { $rutie_c.peek { break } }; $rutie_c.double"),
                10
            );
            assert_eq!(dropped(), before + 1);
        },
    );

    case("a panic is a RuntimeError, after drops", &|| {
        let before = dropped();
        raises(
            "C.new(3).explode",
            "RuntimeError",
            "Rust panic: exploded at 3",
        );
        assert_eq!(dropped(), before + 1);
    });

    case("allocate, uninitialized objects and re-initialize", &|| {
        raises(
            "C.allocate.count",
            "TypeError",
            "uninitialized RutieDerive::Stats::Counter",
        );
        raises(
            "C.new.send(:initialize)",
            "RuntimeError",
            "already initialized",
        );
        raises(
            "RutieDerive::Token.allocate",
            "TypeError",
            "allocator undefined",
        );
        assert_eq!(string("RutieDerive::Greeter.token('t').value"), "t");
    });

    case("dup and clone copy Clone structs", &|| {
        assert_eq!(int("k = RutieDerive::Shape.new(5); k.dup.sides"), 5);
        assert!(eval("s = RutieDerive::Shape.new(3); s.dup.equal?(s)").is_false());
        assert_eq!(
            string("RutieDerive::Square.new(2.0).clone.class.name"),
            "RutieDerive::Square"
        );
        raises(
            "C.new.dup",
            "TypeError",
            "can't copy RutieDerive::Stats::Counter (the Rust struct is not Clone)",
        );
    });

    case("Marshal through _dump and _load", &|| {
        assert_eq!(
            string("Marshal.load(Marshal.dump(RutieDerive::Kit.new)).class.name"),
            "RutieDerive::Kit"
        );
        raises(
            "RutieDerive::Kit._load('nope')",
            "RuntimeError",
            "bad dump: nope",
        );
    });

    case("inheritance between Rust structs", &|| {
        assert_eq!(
            string("RutieDerive::Square.superclass.name"),
            "RutieDerive::Shape"
        );
        assert_eq!(string("RutieDerive::Square.new(3.0).describe"), "4 sides");
        assert_eq!(int("RutieDerive::Square.new(3.0).sides"), 4);
        assert_eq!(string("RutieDerive::Square.new(3.0).area.to_s"), "9.0");
        assert!(
            eval("RutieDerive::Shape.new(4).same_sides(RutieDerive::Square.new(1.0))").is_true()
        );
        raises(
            "RutieDerive::Square.new(1.0).only_squares(RutieDerive::Shape.new(4))",
            "TypeError",
            "expected RutieDerive::Square",
        );
        raises("RutieDerive::Shape.instance_method(:initialize).bind(RutieDerive::Square.allocate).call(4)", "TypeError", "can't initialize a RutieDerive::Square");
        eval("class RutieCube < RutieDerive::Square; def volume; area * length; end; end");
        assert_eq!(string("RutieCube.new(2.0).volume.to_s"), "8.0");
        assert_eq!(string("RutieCube.new(2.0).describe"), "4 sides");
    });

    case("the struct is dropped when Ruby frees the object", &|| {
        let before = dropped();
        eval("100.times { C.new }");
        eval("GC.start(full_mark: true, immediate_sweep: true)");
        assert!(dropped() > before, "no Counter was freed");
    });

    case(
        "fields holding Ruby objects are marked and survive compaction",
        &|| {
            eval("$rutie_store = RutieDerive::Store.new; $rutie_store.fill(200)");
            eval("GC.start(full_mark: true, immediate_sweep: true)");
            eval("GC.verify_compaction_references(expand_heap: true, toward: :empty) rescue GC.compact");
            assert_eq!(int("$rutie_store.check"), 200);
            assert_eq!(int("$rutie_store.dup.check"), 200);
            assert!(int("require 'objspace'; ObjectSpace.memsize_of($rutie_store)") > 0);
        },
    );

    case(
        "a borrowed struct's objects are pinned through compaction",
        &|| {
            eval("$rutie_store2 = RutieDerive::Store.new; $rutie_store2.fill(50)");
            let checked = int(
            "$rutie_store2.check_during { GC.verify_compaction_references(expand_heap: true, toward: :empty) rescue GC.compact }",
        );
            assert_eq!(checked, 50);
            assert_eq!(int("$rutie_store2.check"), 50);
        },
    );

    case("without_gvl lets other Ruby threads run", &|| {
        let ticks = int(
            "ticks = 0; t = Thread.new { 50.times { ticks += 1; sleep 0.002 } }; \
             RutieDerive::Kit.new.sleep_without_gvl(300); t.join; ticks",
        );
        assert_eq!(ticks, 50);
        let during = int(
            "ticks = 0; t = Thread.new { loop { ticks += 1; sleep 0.001 } }; \
             sleep 0.01; before = ticks; RutieDerive::Kit.new.sleep_without_gvl(200); after = ticks; t.kill; after - before",
        );
        assert!(
            during > 10,
            "only {} ticks while the GVL was released",
            during
        );
    });

    case(
        "modules, module functions, constants and nested classes",
        &|| {
            assert_eq!(string("RutieDerive::Greeter.greet('Ruby')"), "Hello, Ruby!");
            assert_eq!(
                string("RutieDerive::Greeter.greet('Ruby', punctuation: '?')"),
                "Hello, Ruby?"
            );
            assert_eq!(string("RutieDerive::Greeter.shout('hi')"), "HI");
            assert!(
                eval("RutieDerive::Greeter.private_instance_methods.include?(:shout)").is_true()
            );
            assert_eq!(string("RutieDerive::Greeter::VERSION"), "1.2.3");
            assert_eq!(int("RutieDerive::Greeter::LIMIT"), 10);
            assert_eq!(string("RutieDerive::Greeter::Card.new('c').text"), "c");
        },
    );

    case("rubyfunction", &|| {
        assert_eq!(string("RutieDerive.standalone('Ruby')"), "Hello, Ruby!");
        assert_eq!(
            string("RutieDerive.standalone('Ruby', '?')"),
            "Hello, Ruby?"
        );
        raises(
            "RutieDerive.standalone(:ruby)",
            "TypeError",
            "Error converting to String",
        );
        assert_eq!(standalone(&RString::new_utf8("Rust"), None), "Hello, Rust!");
    });

    case("RBS declarations", &|| {
        let counter = Counter::rbs();
        let expected = [
            "module RutieDerive",
            "  module Stats",
            "    class Counter",
            "attr_accessor count: Integer",
            "attr_reader label: String",
            "def initialize: (?Integer start, ?name: String) -> void",
            "def add: (Integer amount) -> Integer",
            "def give: (RutieDerive::Stats::Counter other, Integer amount) -> [Integer, Integer]",
            "def each_step: (Integer steps) { (*untyped) -> untyped } -> Integer",
            "def peek: () ?{ (*untyped) -> untyped } -> Integer",
            "def self.origin: () -> RutieDerive::Stats::Counter",
            "def double: () -> Integer",
        ];

        for line in expected {
            assert!(counter.contains(line), "missing {:?} in\n{}", line, counter);
        }

        let kit = Kit::rbs();
        assert!(kit.contains("def options: (Integer first, ?Integer second, *Integer rest, scale: Integer, ?offset: Integer) -> Integer"), "{}", kit);
        assert!(
            Square::rbs().contains("class Square < RutieDerive::Shape"),
            "{}",
            Square::rbs()
        );
        let module = greeter::rbs();
        assert!(module.contains("VERSION: String"), "{}", module);
        assert!(
            module.contains("def self.greet: (String who, ?punctuation: String) -> String"),
            "{}",
            module
        );
        assert!(module.contains("class Card"), "{}", module);
        assert!(
            CounterError::rbs().contains("class OverflowError < RutieDerive::Error"),
            "{}",
            CounterError::rbs()
        );
    });

    case("RubyClass from Rust", &|| {
        let object = Counter::new(None, None).into_object_of(&Counter::class().unwrap());
        assert!(Counter::is_instance(&object));
        assert_eq!(Counter::borrow_from(&object).unwrap().count, 0);
        assert!(!Counter::is_instance(&Symbol::new("nope").to_any_object()));
    });

    println!("1..{} all passed", cases);
}

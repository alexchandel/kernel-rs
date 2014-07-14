#![crate_type = "lib"]
#![feature(asm)]


#[cfg(target_os = "linux", target_arch = "x86")]
#[cfg(target_os = "android", target_arch = "x86")]
pub mod abi
{
	#[inline]
	pub unsafe fn syscall1(num: uint, arg1: i32) -> i32
	{
		let ret: i32;
		asm!("int 0x80": "=0"(ret):
			"0i"(num),
			"b"(arg1)
			::"volatile", "intel");
		ret
	}
}


/// * Push args right-to-left
/// * Align stack to 16 bytes
/// * Set EAX = syscall number
#[cfg(target_os = "macos", target_arch = "x86")]
pub mod abi
{
	#[inline]
	pub unsafe fn syscall0(num: uint) -> i32
	{
		let ret: i32;
		asm!("	mov		eax, $1
				sub		esp, 16
				int		0x80
				add		esp, 16": "=0"(ret):
			""(num)
			:"eax"
			:"volatile", "intel");
		ret
	}


	#[inline]
	pub unsafe fn syscall1(num: uint, arg1: i32) -> i32
	{
		let ret: i32;
		asm!("	push	$2
				mov		eax, $1
				sub		esp, 12
				int		0x80
				add		esp, 16": "=0"(ret):
			""(num),
			""(arg1)
			:"eax"
			:"volatile", "intel");
		ret
	}


	#[inline]
	pub unsafe fn syscall2(num: uint, arg1: i32, arg2: i32) -> i32
	{
		let ret: i32;
		asm!("	push	$3
				push	$2
				mov		eax, $1
				sub		esp, 8
				int		0x80
				add		esp, 16": "=0"(ret):
			""(num),
			""(arg1),
			""(arg2)
			:"eax"
			:"volatile", "intel");
		ret
	}


	#[inline]
	pub unsafe fn syscall3(num: uint, arg1: i32, arg2: i32, arg3: i32) -> i32
	{
		let ret: i32;
		asm!("	push	$4
				push	$3
				push	$2
				mov		eax, $1
				sub		esp, 4
				int		0x80
				add		esp, 16": "=0"(ret):
			""(num),
			""(arg1),
			""(arg2),
			""(arg3)
			:"eax"
			:"volatile", "intel");
		ret
	}


	#[inline]
	pub unsafe fn syscall3(num: uint, arg1: i32, arg2: i32, arg3: i32, arg4: i32) -> i32
	{
		let ret: i32;
		asm!("	push	$5
				push	$4
				push	$3
				push	$2
				mov		eax, $1
				int		0x80
				add		esp, 16": "=0"(ret):
			""(num),
			""(arg1),
			""(arg2),
			""(arg3),
			""(arg4)
			:"eax"
			:"volatile", "intel");
		ret
	}
}


/// * Align stack to 16 bytes
/// * Set RAX = syscall number
/// rax = syscall
/// rdi = arg1
/// rsi = arg2
/// rdx = arg3
/// r10 = arg4        // rcx on 10.5 or sooner, just like user space
/// r8  = arg5
/// r9  = arg6
/// kernel destroys registers %rcx and %r11.
/// syscall has to be passed in register %rax.
/// limited to six arguments, no argument is passed directly on the stack.
/// %rax contains the result, -4095 and -1 indicates an error

#[cfg(unix, target_arch = "x86_64")]
pub mod abi
{
	#[inline]
	pub unsafe fn syscall0(num: uint) -> i64
	{
		let ret: i64;
		asm!("syscall": "={rax}"(ret):
			"{rax}"(num)
			::"volatile", "intel");
		ret
		// if ret >= 0 {Ok(ret)} else {Err(ret)}
	}


	#[inline]
	pub unsafe fn syscall1(num: uint, arg1: i64) -> i64
	{
		let ret: i64;
		asm!("syscall": "={rax}"(ret):
			"{rax}"(num),
			"{rdi}"(arg1)
			::"volatile", "intel");
		ret
		// if ret >= 0 {Ok(ret)} else {Err(ret)}
	}


	#[inline]
	pub unsafe fn syscall2(num: uint, arg1: i64, arg2: i64) -> i64
	{
		let mut ret: i64;
		asm!("syscall": "={rax}"(ret):
			"{rax}"(num),
			"{rdi}"(arg1),
			"{rsi}"(arg2)
			::"volatile", "intel");
		ret
	}


	#[inline]
	pub unsafe fn syscall3(num: uint, arg1: i64, arg2: i64, arg3: i64) -> i64
	{
		let mut ret: i64;
		asm!("syscall": "={rax}"(ret):
			"{rax}"(num),
			"{rdi}"(arg1),
			"{rsi}"(arg2),
			"{rdx}"(arg3)
			::"volatile", "intel");
		ret
	}
}


#[cfg(target_os = "freebsd")]
#[cfg(target_os = "linux")]
#[cfg(target_os = "android")]
pub mod sys
{
	use abi::syscall1;
	use abi::syscall2;
	use abi::syscall3;


	#[inline]
	pub fn exit(ret: i32)
	{
		unsafe
		{
			syscall1(60, ret);
		}
	}


	#[inline]
	pub unsafe fn getcwd(buffer: *mut u8, size: uint) -> i64
	{
		syscall2(79, buffer as i64, size as i64)
	}
}


#[cfg(target_os = "macos", target_arch = "x86_64")]
pub mod sys
{
	use abi::syscall0;
	use abi::syscall1;
	use abi::syscall2;
	use abi::syscall3;


	/// Exits with return value.
	#[inline]
	pub fn exit(ret: i64)
	{
		unsafe
		{
			syscall1(1 + 0x2000000, ret);
		}
	}


	#[inline]
	pub fn fork() -> i64
	{
		unsafe
		{
			syscall0(2 + 0x2000000)
		}
	}


	/// Writes nbytes from source to the file descriptor fd. Returns the
	/// number of bytes writen.
	#[inline]
	pub unsafe fn write(fd: uint, source: *const u8, nbytes: uint) -> int
	{
		syscall3(4 + 0x2000000, fd as i64, source as i64, nbytes as i64) as int
	}


	#[inline]
	pub fn write_str(fd: uint, msg: &str) -> int
	{
		unsafe
		{
			write(fd, msg.as_ptr(), msg.len())
		}
	}
}


fn main() {
	let msg: &'static str = "Hello\n";
	let ret: int = sys::write_str(1, msg);
	println!("{0}", ret);
}




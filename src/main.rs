use std::io::{Read,Write};

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
enum Instruction{
	MoveRight,
	MoveLeft,
	Increment,
	Decrement,
	Write,
	Read,
	OpenLoop(usize),
	CloseLoop(usize),
}
#[derive(Debug)]
pub enum LexError{
	InvalidInstruction(u8),
	UnmatchedOpenLoop{position:usize},
	UnmatchedCloseLoop{position:usize},
}

#[derive(Debug)]
struct Brainfuck{
	instructions:Vec<Instruction>,
	instruction_head:usize,
	data:Vec<u8>,
	data_head:usize,
}
impl TryFrom<&[u8]> for Brainfuck{
	type Error=LexError;
	fn try_from(value:&[u8])->Result<Self,Self::Error>{
		let mut stack=Vec::new();
		let mut first_pass=Vec::with_capacity(value.len());
		for (i,byte) in value.iter().enumerate(){
			let ins=match byte{
				b'>'=>Some(Instruction::MoveRight),
				b'<'=>Some(Instruction::MoveLeft),
				b'+'=>Some(Instruction::Increment),
				b'-'=>Some(Instruction::Decrement),
				b'.'=>Some(Instruction::Write),
				b','=>Some(Instruction::Read),
				b'['=>{stack.push(i);None},
				b']'=>{
					let open_loop_index=stack.pop().ok_or(LexError::UnmatchedCloseLoop{position:i})?;
					*first_pass.get_mut(open_loop_index).unwrap()=Some(Instruction::OpenLoop(i));
					Some(Instruction::CloseLoop(open_loop_index))
				},
				&other=>return Err(LexError::InvalidInstruction(other)),
			};
			first_pass.push(ins);
		}
		Ok(Brainfuck{
			instructions:first_pass.into_iter().collect::<Option<Vec<_>>>().ok_or_else(||LexError::UnmatchedOpenLoop{position:stack[0]})?,
			instruction_head:0,
			data:Vec::new(),
			data_head:0,
		})
	}
}
impl TryFrom<&str> for Brainfuck{
	type Error=LexError;
	fn try_from(value:&str)->Result<Self,Self::Error>{
		Self::try_from(value.as_bytes())
	}
}
#[derive(Debug)]
pub enum RunError{
	Io(std::io::Error),
	PointerOverflow{position:usize},
	PointerUnderflow{position:usize},
}
impl Brainfuck{
	fn get_or_reserve(&mut self)->&mut u8{
		if self.data.len()<=self.data_head{
			self.data.extend((self.data.len()..=self.data_head).map(|_|0));
		}
		self.data.get_mut(self.data_head).unwrap()
	}
	/// Returns Result<should_continue>
	fn step(&mut self)->Result<bool,RunError>{
		let ins=self.instructions.get(self.instruction_head);
		match ins{
			Some(Instruction::MoveRight)=>match self.data_head.checked_add(1){
				Some(value)=>self.data_head=value,
				None=>return Err(RunError::PointerOverflow{position:self.instruction_head}),
			},
			Some(Instruction::MoveLeft)=>match self.data_head.checked_sub(1){
				Some(value)=>self.data_head=value,
				None=>return Err(RunError::PointerUnderflow{position:self.instruction_head}),
			},
			Some(Instruction::Increment)=>{
				let c=self.get_or_reserve();
				*c=c.wrapping_add(1);
			},
			Some(Instruction::Decrement)=>{
				let c=self.get_or_reserve();
				*c=c.wrapping_sub(1);
			},
			Some(Instruction::Write)=>{
				std::io::stdout().write(std::slice::from_ref(self.get_or_reserve())).map_err(RunError::Io)?;
			},
			Some(Instruction::Read)=>{
				std::io::stdin().read_exact(std::slice::from_mut(self.get_or_reserve())).map_err(RunError::Io)?;
			},
			Some(&Instruction::OpenLoop(index))=>if *self.get_or_reserve()==0{
				self.instruction_head=index;
			},
			Some(&Instruction::CloseLoop(index))=>if *self.get_or_reserve()!=0{
				self.instruction_head=index;
			},
			None=>return Ok(false),
		}
		self.instruction_head+=1;
		Ok(true)
	}
	fn run(&mut self)->Result<(),RunError>{
		while self.step()?{}
		std::io::stdout().flush().map_err(RunError::Io)?;
		Ok(())
	}
}


fn main(){
	println!("Example1: Underflow");
	println!("Result={:?}",Brainfuck::try_from(
		"+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+."
	).unwrap().run());

	println!("Example2: Unmatched loop");
	println!("Result={:?}",Brainfuck::try_from(
		"+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>"
	));

	println!("Example3:");
	Brainfuck::try_from(
		"++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++."
	).unwrap().run().unwrap();
}

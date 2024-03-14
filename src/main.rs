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
enum Error{
	InvalidInstruction(u8),
	UnmatchedOpenLoop(usize),
	UnmatchedCloseLoop(usize),
}

#[derive(Debug)]
struct Brainfuck{
	instructions:Vec<Instruction>,
	instruction_head:usize,
	data:Vec<u8>,
	data_head:usize,
}
impl TryFrom<&str> for Brainfuck{
	type Error=Error;
	fn try_from(value:&str)->Result<Self,Self::Error>{
		let mut stack=Vec::new();
		let mut first_pass=Vec::with_capacity(value.len());
		for (i,byte) in value.bytes().enumerate(){
			let ins=match byte{
				b'>'=>Some(Instruction::MoveRight),
				b'<'=>Some(Instruction::MoveLeft),
				b'+'=>Some(Instruction::Increment),
				b'-'=>Some(Instruction::Decrement),
				b'.'=>Some(Instruction::Write),
				b','=>Some(Instruction::Read),
				b'['=>{stack.push(i);None},
				b']'=>{
					let open_loop_index=stack.pop().ok_or(Error::UnmatchedCloseLoop(i))?;
					*first_pass.get_mut(open_loop_index).unwrap()=Some(Instruction::OpenLoop(i));
					Some(Instruction::CloseLoop(open_loop_index))
				},
				other=>return Err(Error::InvalidInstruction(other)),
			};
			first_pass.push(ins);
		}
		Ok(Brainfuck{
			instructions:first_pass.into_iter().collect::<Option<Vec<_>>>().ok_or_else(||Error::UnmatchedOpenLoop(stack.pop().unwrap()))?,
			instruction_head:0,
			data:Vec::new(),
			data_head:0,
		})
	}
}
impl Brainfuck{
	fn get_or_reserve(&mut self)->&mut u8{
		if self.data.len()<=self.data_head{
			self.data.extend((self.data.len()..=self.data_head).map(|_|0));
		}
		self.data.get_mut(self.data_head).unwrap()
	}
	fn step(&mut self)->bool{
		let ins=self.instructions.get(self.instruction_head);
		match ins{
			Some(Instruction::MoveRight)=>self.data_head+=1,
			Some(Instruction::MoveLeft)=>self.data_head-=1,
			Some(Instruction::Increment)=>{
				let c=self.get_or_reserve();
				*c=c.wrapping_add(1);
			},
			Some(Instruction::Decrement)=>{
				let c=self.get_or_reserve();
				*c=c.wrapping_sub(1);
			},
			Some(Instruction::Write)=>{
				std::io::stdout().write(std::slice::from_ref(self.get_or_reserve())).unwrap();
			},
			Some(Instruction::Read)=>{
				std::io::stdin().read_exact(std::slice::from_mut(self.get_or_reserve())).unwrap();
			},
			Some(&Instruction::OpenLoop(index))=>if *self.get_or_reserve()==0{
				self.instruction_head=index;
			},
			Some(&Instruction::CloseLoop(index))=>if *self.get_or_reserve()!=0{
				self.instruction_head=index;
			},
			None=>return false,
		}
		self.instruction_head+=1;
		true
	}
	fn run(&mut self){
		while self.step(){}
		std::io::stdout().flush().unwrap();
	}
}


fn main(){
	let mut bf=Brainfuck::try_from("++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.").unwrap();
	bf.run();
}

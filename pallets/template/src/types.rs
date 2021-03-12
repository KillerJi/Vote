use codec::{Encode, Decode};
use sp_runtime::RuntimeDebug;
use sp_runtime::traits::Zero;



#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum VotingOptions{
	A,
	B,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct VotingInfo<BlockNumber,Balance>{
	pub (crate) VoteStatus: VotingStatus,// 投票是否完成
	pub (crate) VoteNumber: VotingNumber<Balance>,// 投票数量 A B
	pub (crate) MajorityVote: Option<VotingOptions>,// 当前多数票
	pub (crate) BeganBlock: BlockNumber,// 开始区块
	pub (crate) FinishedBlock: BlockNumber, // 结束区块
}
impl <BlockNumber:Ord + Copy + Zero, Balance:Ord + Copy + Zero + Default> VotingInfo<BlockNumber,Balance>{
	pub fn new(since:BlockNumber, end: BlockNumber) -> Self{
		VotingInfo{
			VoteStatus: VotingStatus::Ongoing,
			VoteNumber: VotingNumber::default(),
			MajorityVote: None,
			BeganBlock: since,
			FinishedBlock: end,
		}
	}
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum VotingStatus{
	Ongoing  ,// 正在进行 投票数量 当前区块
	Finished ,          // 已完成   投票结果 结束区块
}

// impl <BlockNumber:Ord + Copy + Zero, Balance:Ord + Copy + Zero + Default> VotingStatus<BlockNumber,Balance>{
// 	fn default(since:BlockNumber) -> Self{
// 		VotingStatus::Ongoing{
// 			VotingNumber: VotingNumber::default(),
// 			CurrentBlock: since,
// 		}
// 	}
	
// }

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, Default)]
pub struct VotingNumber<Balance>{
	pub (crate) A: Balance,// 投票数量
	pub (crate) B: Balance,// 投票数量
}

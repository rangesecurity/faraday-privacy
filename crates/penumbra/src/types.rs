use penumbra_sdk_transaction::view::action_view::ActionView;

/// New type helper used to parse [`penumbra_sdk_transaction::view::action_view::ActionView`] into a String
pub struct TransactionType(String);

impl From<&ActionView> for TransactionType {
    fn from(value: &ActionView) -> Self {
        let name = match value {
            ActionView::Spend(..) => "Spend",
            ActionView::Output(..) => "Output",
            ActionView::Swap(..) => "Swap",
            ActionView::SwapClaim(..) => "SwapClaim",
            ActionView::DelegatorVote(..) => "DelegatorVote",
            ActionView::ValidatorDefinition(..) => "ValidatorDefinition",
            ActionView::IbcRelay(..) => "IbcRelay",
            ActionView::ProposalSubmit(..) => "ProposalSubmit",
            ActionView::ProposalWithdraw(..) => "ProposalWithdraw",
            ActionView::ValidatorVote(..) => "ValidatorVote",
            ActionView::ProposalDepositClaim(..) => "ProposalDepositClaim",
            ActionView::PositionOpen(..) => "PositionOpen",
            ActionView::PositionClose(..) => "PositionClose",
            ActionView::PositionWithdraw(..) => "PositionWithdraw",
            ActionView::Delegate(..) => "Delegate",
            ActionView::Undelegate(..) => "Undelegate",
            ActionView::UndelegateClaim(..) => "UndelegateClaim",
            ActionView::Ics20Withdrawal(..) => "Ics20Withdrawal",
            ActionView::CommunityPoolDeposit(..) => "CommunityPoolDeposit",
            ActionView::CommunityPoolSpend(..) => "CommunityPoolSpend",
            ActionView::CommunityPoolOutput(..) => "CommunityPoolOutput",
            ActionView::ActionDutchAuctionSchedule(..) => "ActionDutchAuctionSchedule",
            ActionView::ActionDutchAuctionEnd(..) => "ActionDutchAuctionEnd",
            ActionView::ActionDutchAuctionWithdraw(..) => "ActionDutchAuctionWithdraw",
        };

        Self(name.to_string())
    }
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<String> for TransactionType {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

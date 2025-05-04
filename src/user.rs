#![allow(unused)]

#[derive(Debug, Default)]
pub struct User {
   pub id: u32,
   pub name: String,
   pub total_deposited: u32,
   pub total_withdrawn: u32,
   pub has_deposited: bool,
   pub borrowable: bool,
}

#[derive(Debug, Default)]
pub struct Treasury {
   pub sum_deposited: u32,
   pub sum_withdrawn: u32,
}

impl User {
    /// Deposit `amount` into the user’s account and the treasury.
    pub fn deposit(&mut self, amount: u32, treasury: &mut Treasury, is_borrowable: bool) {
        self.total_deposited = self
            .total_deposited
            .checked_add(amount)
            .expect("deposit overflow");
        self.has_deposited = true;
        self.borrowable = is_borrowable;
        treasury.sum_deposited = treasury
            .sum_deposited
            .checked_add(amount)
            .expect("treasury deposit overflow");
    }

    /// Withdraw `amount` from the user’s account and the treasury.
    pub fn withdraw(
        &mut self,
        amount: u32,
        treasury: &mut Treasury,
    ) -> Result<u32, String> {
        if self.total_withdrawn + amount <= self.total_deposited {
            // Deduct from deposited balance
            self.total_deposited -= amount;
            self.total_withdrawn = self
                .total_withdrawn
                .checked_add(amount)
                .expect("withdraw overflow");
            // Adjust treasury
            treasury.sum_deposited -= amount;
            treasury.sum_withdrawn = treasury
                .sum_withdrawn
                .checked_add(amount)
                .expect("treasury withdrawal overflow");
            Ok(self.total_withdrawn)
        } else {
            Err(String::from("Something Went Wrong"))
        }
    }

    /// Calculate the entry fee (2% fee) for the given deposit amount.
    pub fn calculate_entry_fee(amount: u32) -> u32 {
        const MAX_BPS: u32 = 10_000;
        const ENTRY_FEE_BPS: u32 = 200; // 2%
        (amount.saturating_mul(ENTRY_FEE_BPS)) / MAX_BPS
    }

    /// Calculate the exit fee (4% fee) for the given withdrawal amount.
    pub fn calculate_exit_fee(amount: u32) -> u32 {
        const MAX_BPS: u32 = 10_000;
        const EXIT_FEE_BPS: u32 = 400; // 4%
        (amount.saturating_mul(EXIT_FEE_BPS)) / MAX_BPS
    }

    /// Deposit with an entry fee deducted.
    /// The net deposit (amount minus fee) is credited into the user's account.
    pub fn deposit_with_fee(&mut self, amount: u32, treasury: &mut Treasury, is_borrowable: bool) {
        let fee = Self::calculate_entry_fee(amount);
        let net_amount = amount.checked_sub(fee)
            .expect("Fee exceeds deposit amount");
        self.deposit(net_amount, treasury, is_borrowable);
        // Optionally, you might record the fee separately.
    }

    /// Withdraw funds along with an exit fee.
    /// The total withdrawal is the requested amount plus the fee.
    pub fn withdraw_with_fee(&mut self, amount: u32, treasury: &mut Treasury) -> Result<u32, String> {
        let fee = Self::calculate_exit_fee(amount);
        let total = amount.checked_add(fee)
            .ok_or("Withdrawal fee calculation error")?;
        self.withdraw(total, treasury)
    }

    /// Borrow funds from a lender.
    /// The borrower is allowed to borrow up to 10% of the lender's deposited funds,
    /// provided the lender has enabled borrowing.
    pub fn borrow(&mut self, lender: &mut User, amount: u32) -> Result<u32, String> {
        const BORROW_PERCENTAGE: u32 = 10; // 10% borrowing limit
        
        if !lender.borrowable {
            return Err(String::from("Lender has not enabled borrowing"));
        }

        // Calculate maximum borrowable amount (10% of lender's deposited amount)
        let max_borrowable = (lender.total_deposited * BORROW_PERCENTAGE) / 100;
        
        if amount > max_borrowable {
            return Err(format!(
                "Cannot borrow more than {}% of lender's deposit. Maximum: {}", 
                BORROW_PERCENTAGE, 
                max_borrowable
            ));
        }

        if lender.total_deposited < amount {
            return Err(String::from("Insufficient funds in lender's account"));
        }

        // Update balances
        lender.total_deposited = lender.total_deposited
            .checked_sub(amount)
            .ok_or("Arithmetic overflow")?;

        self.total_deposited = self.total_deposited
            .checked_add(amount)
            .ok_or("Arithmetic overflow")?;

        Ok(amount)
    }
}

impl Treasury {
    /// Calculate the interest rate and apply interest to the user's deposit.
    /// Returns the interest amount applied.
    pub fn apply_interest(&mut self, user: &mut User) -> Result<u32, String> {
        let interest = Self::calculate_interest_rate(self, user)?;
        user.total_deposited = user.total_deposited
            .checked_add(interest)
            .ok_or("Arithmetic overflow when applying interest")?;
        self.sum_deposited = self.sum_deposited
            .checked_add(interest)
            .ok_or("Arithmetic overflow when applying interest to treasury")?;
        Ok(interest)
    }
    
    /// Calculate interest rate based on treasury and user's deposit.
    /// Returns `interest = (treasury.sum_deposited * user.total_deposited) / treasury.sum_withdrawn`
    /// or an error if the treasury state is invalid.
    pub fn calculate_interest_rate(treasury: &Treasury, user: &User) -> Result<u32, String> {
        if treasury.sum_deposited > 0 && treasury.sum_withdrawn > 0 {
            Ok((treasury.sum_deposited.saturating_mul(user.total_deposited)) / treasury.sum_withdrawn)
        } else {
            Err(String::from("Invalid treasury state"))
        }
    }
}

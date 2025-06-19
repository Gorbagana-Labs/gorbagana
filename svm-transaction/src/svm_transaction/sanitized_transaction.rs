use {
    crate::svm_transaction::SVMTransaction, gorbagana_signature::Signature,
    gorbagana_transaction::sanitized::SanitizedTransaction,
};

impl SVMTransaction for SanitizedTransaction {
    fn signature(&self) -> &Signature {
        SanitizedTransaction::signature(self)
    }

    fn signatures(&self) -> &[Signature] {
        SanitizedTransaction::signatures(self)
    }
}

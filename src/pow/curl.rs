use super::sponge::{Mode, Sponge, HASH_LENGTH};
use crate::utils::converter::array_copy;
use failure::Error;

const STATE_LENGTH: usize = 3 * HASH_LENGTH;
const TRUTH_TABLE: [i8; 11] = [1, 0, -1, 2, 1, -1, 0, 2, -1, 1, 0];

#[derive(Clone, Copy)]
pub struct Curl {
    number_of_rounds: i32,
    scratchpad: [i8; STATE_LENGTH],
    state: [i8; STATE_LENGTH],
}

impl Default for Curl {
    fn default() -> Curl {
        Curl {
            number_of_rounds: 81,
            scratchpad: [0; STATE_LENGTH],
            state: [0; STATE_LENGTH],
        }
    }
}

impl Curl {
    pub fn new(mode: Mode) -> Result<Curl, Error> {
        let mut curl = Curl::default();
        curl.number_of_rounds = match mode {
            Mode::CURLP27 => 27,
            Mode::CURLP81 => 81,
            a => return Err(format_err!("Invalid mode: {}", a)),
        };
        Ok(curl)
    }

    fn transform(&mut self) {
        let mut scratchpad_index = 0;
        for _ in 0..self.number_of_rounds {
            array_copy(&self.state, 0, &mut self.scratchpad, 0, STATE_LENGTH);
            for state_index in 0..STATE_LENGTH {
                let prev_scratchpad_index = scratchpad_index;
                if scratchpad_index < 365 {
                    scratchpad_index += 364;
                } else {
                    scratchpad_index -= 365;
                }
                let truth_index = (self.scratchpad[prev_scratchpad_index]
                    + (self.scratchpad[scratchpad_index] << 2)
                    + 5) as usize;
                self.state[state_index] = TRUTH_TABLE[truth_index];
            }
        }
    }

    fn state(&self) -> &[i8] {
        &self.state
    }

    fn state_mut(&mut self) -> &mut [i8] {
        &mut self.state
    }
}

impl Sponge for Curl {
    fn absorb(&mut self, trits: &mut [i8]) {
        for chunk in trits.chunks(HASH_LENGTH) {
            self.state[0..HASH_LENGTH].clone_from_slice(chunk);
            self.transform();
        }
    }

    fn squeeze(&mut self, out: &mut [i8]) {
        let trit_length = out.len();
        let hash_length = trit_length / HASH_LENGTH;

        for chunk in out.chunks_mut(HASH_LENGTH) {
            chunk.clone_from_slice(&self.state[0..HASH_LENGTH]);
            self.transform();
        }

        let last = trit_length - hash_length * HASH_LENGTH;
        out[trit_length - last..].clone_from_slice(&self.state[0..last]);
        if trit_length % HASH_LENGTH != 0 {
            self.transform();
        }
    }

    fn reset(&mut self) {
        self.state = [0; STATE_LENGTH];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::converter;

    const TRYTES: &str = "RSWWSFXPQJUBJROQBRQZWZXZJWMUBVIVMHPPTYSNW9YQIQQF9RCSJJCVZG9ZWITXNCSBBDHEEKDRBHVTWCZ9SZOOZHVBPCQNPKTWFNZAWGCZ9QDIMKRVINMIRZBPKRKQAIPGOHBTHTGYXTBJLSURDSPEOJ9UKJECUKCCPVIQQHDUYKVKISCEIEGVOQWRBAYXWGSJUTEVG9RPQLPTKYCRAJ9YNCUMDVDYDQCKRJOAPXCSUDAJGETALJINHEVNAARIPONBWXUOQUFGNOCUSSLYWKOZMZUKLNITZIFXFWQAYVJCVMDTRSHORGNSTKX9Z9DLWNHZSMNOYTU9AUCGYBVIITEPEKIXBCOFCMQPBGXYJKSHPXNUKFTXIJVYRFILAVXEWTUICZCYYPCEHNTK9SLGVL9RLAMYTAEPONCBHDXSEQZOXO9XCFUCPPMKEBR9IEJGQOPPILHFXHMIULJYXZJASQEGCQDVYFOM9ETXAGVMSCHHQLFPATWOSMZIDL9AHMSDCE9UENACG9OVFAEIPPQYBCLXDMXXA9UBJFQQBCYKETPNKHNOUKCSSYLWZDLKUARXNVKKKHNRBVSTVKQCZL9RY9BDTDTPUTFUBGRMSTOTXLWUHDMSGYRDSZLIPGQXIDMNCNBOAOI9WFUCXSRLJFIVTIPIAZUK9EDUJJ9B9YCJEZQQELLHVCWDNRH9FUXDGZRGOVXGOKORTCQQA9JXNROLETYCNLRMBGXBL9DQKMOAZCBJGWLNJLGRSTYBKLGFVRUF9QOPZVQFGMDJA9TBVGFJDBAHEVOLW9GNU9NICLCQJBOAJBAHHBZJGOFUCQMBGYQLCWNKSZPPBQMSJTJLM9GXOZHTNDLGIRCSIJAZTENQVQDHFSOQM9WVNWQQJNOPZMEISSCLOADMRNWALBBSLSWNCTOSNHNLWZBVCFIOGFPCPRKQSRGKFXGTWUSCPZSKQNLQJGKDLOXSBJMEHQPDZGSENUKWAHRNONDTBLHNAKGLOMCFYRCGMDOVANPFHMQRFCZIQHCGVORJJNYMTORDKPJPLA9LWAKAWXLIFEVLKHRKCDG9QPQCPGVKIVBENQJTJGZKFTNZHIMQISVBNLHAYSSVJKTIELGTETKPVRQXNAPWOBGQGFRMMK9UQDWJHSQMYQQTCBMVQKUVGJEAGTEQDN9TCRRAZHDPSPIYVNKPGJSJZASZQBM9WXEDWGAOQPPZFLAMZLEZGXPYSOJRWL9ZH9NOJTUKXNTCRRDO9GKULXBAVDRIZBOKJYVJUSHIX9F9O9ACYCAHUKBIEPVZWVJAJGSDQNZNWLIWVSKFJUMOYDMVUFLUXT9CEQEVRFBJVPCTJQCORM9JHLYFSMUVMFDXZFNCUFZZIKREIUIHUSHRPPOUKGFKWX9COXBAZMQBBFRFIBGEAVKBWKNTBMLPHLOUYOXPIQIZQWGOVUWQABTJT9ZZPNBABQFYRCQLXDHDEX9PULVTCQLWPTJLRSVZQEEYVBVY9KCNEZXQLEGADSTJBYOXEVGVTUFKNCNWMEDKDUMTKCMRPGKDCCBDHDVVSMPOPUBZOMZTXJSQNVVGXNPPBVSBL9WWXWQNMHRMQFEQYKWNCSW9URI9FYPT9UZMAFMMGUKFYTWPCQKVJ9DIHRJFMXRZUGI9TMTFUQHGXNBITDSORZORQIAMKY9VRYKLEHNRNFSEFBHF9KXIQAEZEJNQOENJVMWLMHI9GNZPXYUIFAJIVCLAGKUZIKTJKGNQVTXJORWIQDHUPBBPPYOUPFAABBVMMYATXERQHPECDVYGWDGXFJKOMOBXKRZD9MCQ9LGDGGGMYGUAFGMQTUHZOAPLKPNPCIKUNEMQIZOCM9COAOMZSJ9GVWZBZYXMCNALENZ9PRYMHENPWGKX9ULUIGJUJRKFJPBTTHCRZQKEAHT9DC9GSWQEGDTZFHACZMLFYDVOWZADBNMEM9XXEOMHCNJMDSUAJRQTBUWKJF9RZHK9ACGUNI9URFIHLXBXCEODONPXBSCWP9WNAEYNALKQHGULUQGAFL9LB9NBLLCACLQFGQMXRHGBTMI9YKAJKVELRWWKJAPKMSYMJTDYMZ9PJEEYIRXRMMFLRSFSHIXUL9NEJABLRUGHJFL9RASMSKOI9VCFRZ9GWTMODUUESIJBHWWHZYCLDENBFSJQPIOYC9MBGOOXSWEMLVU9L9WJXKZKVDBDMFSVHHISSSNILUMWULMVMESQUIHDGBDXROXGH9MTNFSLWJZRAPOKKRGXAAQBFPYPAAXLSTMNSNDTTJQSDQORNJS9BBGQ9KQJZYPAQ9JYQZJ9B9KQDAXUACZWRUNGMBOQLQZUHFNCKVQGORRZGAHES9PWJUKZWUJSBMNZFILBNBQQKLXITCTQDDBV9UDAOQOUPWMXTXWFWVMCXIXLRMRWMAYYQJPCEAAOFEOGZQMEDAGYGCTKUJBS9AGEXJAFHWWDZRYEN9DN9HVCMLFURISLYSWKXHJKXMHUWZXUQARMYPGKRKQMHVR9JEYXJRPNZINYNCGZHHUNHBAIJHLYZIZGGIDFWVNXZQADLEDJFTIUTQWCQSX9QNGUZXGXJYUUTFSZPQKXBA9DFRQRLTLUJENKESDGTZRGRSLTNYTITXRXRGVLWBTEWPJXZYLGHLQBAVYVOSABIVTQYQM9FIQKCBRRUEMVVTMERLWOK";
    const HASH: &str =
        "TIXEPIEYMGURTQ9ABVYVQSWMNGCVQFASMFAEQWUZCLIWLCDIGYVXOEJBBEMZOIHAYSUQMEFOGZBXUMHQW";

    #[test]
    fn test_curl_works() {
        let size = 8019;
        let mut in_trits = converter::trits_from_string(TRYTES);
        let mut hash_trits = vec![0; HASH_LENGTH];
        let mut curl = Curl::default();
        curl.absorb(&mut in_trits[0..size]);
        curl.squeeze(&mut hash_trits);
        let out_trytes = converter::trytes(&hash_trits);
        assert_eq!(HASH, out_trytes);
    }
}

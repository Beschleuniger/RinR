use serenity::{model::prelude::{Member, RoleId}, prelude::{Context, SerenityError}};

const ID_MORITZ: u64 =                 366590223196356608;
const ID_DANA: u64 =                   204281179636105226;
const ID_FLO: u64 =                    389063964836888579;
const ID_VALI: u64 =                   299215764920205314;

const ID_DANA_PHONE: u64 =             759736758534668348;
const ID_FLO_PHONE: u64 =              891006371967926282;
const ID_MASL_PHONE: u64 =            1080477409658290186;

const ROLE_REINCARNATION: RoleId =     RoleId(896806202657341451);
const ROLE_MOBILE: RoleId =            RoleId(1047203633080586340);
const ROLE_GREATER_YIKE: RoleId =      RoleId(1047202830487928853);
const ROLE_LESSER_YIKES: RoleId =      RoleId(1047201004560592906);
const ROLE_REFUGEE: RoleId =           RoleId(905199210809397259);


pub async fn resolveRoles(ctx: Context, member: &mut Member) -> Result<(), SerenityError> {

    let member_id: u64 = member.user.id.0;

    let mut role_ids: Vec<RoleId> = Vec::with_capacity(2);

     match member_id {
        ID_DANA => {
            role_ids.push(ROLE_REINCARNATION);
            role_ids.push(ROLE_GREATER_YIKE);
        },
        ID_MORITZ => role_ids.push(ROLE_GREATER_YIKE),
        ID_DANA_PHONE | ID_FLO_PHONE | ID_MASL_PHONE => role_ids.push(ROLE_MOBILE),
        ID_FLO | ID_VALI => role_ids.push(ROLE_LESSER_YIKES),
        _ => role_ids.push(ROLE_REFUGEE),
    }

    let success: Result<Vec<RoleId>, SerenityError> = member.add_roles(ctx.http, role_ids.as_slice()).await;

    let ret: Result<(), SerenityError> = match success {
        Ok(_) => Ok(()),
        Err(E) => Err(E),
    }; 

    ret
}
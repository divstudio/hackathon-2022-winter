import request from "../request/index";
import http from "../request/index";
// import $store from "@/store/index";

// export function getGovernanceList() {
//   let arr = [
//     "query {",
//     "    startGoverns (first: 5) {",
//     "        nodes {",
//     "            id",
//     "            creator",
//     "            number",
//     "            governType",
//     "           startDate",
//     "   endDate",
//     "  uintValue",
//     " totalVoter",
//     "        }",
//     "    }",
//     "    voteByNumbers (first: 5) {",
//     "        nodes {",
//     "            id",
//     "            votor",
//     "           voters",
//     "            number",
//     "            governType",
//     "        }",
//     "    }",
//     "}",
//   ];
//   return request({
//     url: "/thiscrg/stafidao",
//     method: "post",
//     data: { query: arr.join(), variables: null },
//   });
// }

// export function getAirdropList() {
//   let arr = [
//     "query {",
//     "    dropings (",
//     "      filter:{",
//     '        to:{equalToInsensitive:"' + $store.state.accs + '"}',
//     "      }",
//     "      first: 5) {",
//     "        nodes {",
//     "            id",
//     "            value",
//     "            to",
//     "            from",
//     "            contractAddress",
//     "        }",
//     "    }",
//     "}",
//   ];

//   return request({
//     url: "/makefriendwithtime/stafidao-airdrop",
//     method: "post",
//     data: { query: arr.join() },
//   });
// }

// export function getRewardList() {
//   let arr = [
//     "query {",
//     "    assignRewards (",
//     "       filter:{",
//     '       recipient :{equalToInsensitive:"' + $store.state.accs + '"}',
//     "      }",
//     "      first: 5) {",
//     "        nodes {",
//     "            id",
//     "            recipient",
//     "            amount",
//     "            contractAddress",
//     "        }",
//     "    }",
//     "}",
//   ];
//   return request({
//     url: "/makefriendwithtime/stafidao-reward",
//     method: "post",
//     data: { query: arr.join() },
//   });
// }

// export default function getList() {
//   return request({
//     url: "/api/productlist",
//     method: "get",
//   });
// }

// function getList() {
//   let arr = [
//     "query {",
//     "    dropings (",
//     "      filter:{",
//     '        to:{equalToInsensitive:"0x1EDbd6d0beEC7c372AEeF8fc5d3b3706043e3B0B"}',
//     "      }",
//     "      first: 5) {",
//     "        nodes {",
//     "            id",
//     "            value",
//     "            to",
//     "            from",
//     "            contractAddress",
//     "        }",
//     "    }",
//     "}",
//   ];
//   return new Promise((resolve, reject) => {
//     http("post", "/makefriendwithtime/stafidao-airdrop", arr.join()).then(
//       (res) => {
//         resolve(res);
//       },
//       (error) => {
//         console.log("网络异常~", error);
//         reject(error);
//       }
//     );
//   });
// }
function getList() {
  let arr = [
    "query {",
    "contents(filter:{flag:{equalTo:true}},first: 10,offset:0,orderBy:BLOCK_HASH_ASC){",
    "      nodes{",
    "        id,",
    "     blockHash,",
    "     content,",
    "        categoryId",
    "     category{",
    "          name",
    "        },",
    "     label,",
    "        dimensionId,",
    "     dimension{",
    "          name",
    "        },",
    "     lastAuthor,",
    "     lastDate,",
    "     flag",
    "      }",
    "    }",
    "}",
  ];
  return new Promise((resolve, reject) => {
    http("post", "", { query: arr.join() }).then(
      (res) => {
        resolve(res);
      },
      (error) => {
        console.log("网络异常~", error);
        reject(error);
      }
    );
  });
}
export { getList };

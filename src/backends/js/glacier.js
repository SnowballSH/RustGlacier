const print = console.log;

let $Q = [];
const $P = () => {
    let res = $Q.pop();
    return res !== undefined ? res : null;
};
const $U = t => $Q.push(t);

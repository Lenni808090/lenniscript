async function test(num1, num2) {
    try {
        let res = (num1 + num2)
        return res
        for (let i = 0; (i < 10); ++i) {
            console.log("hi")
            if ((i == 6)) {
                break
            }
        }
        let zahl = await (5 + 5)
    } catch {
        console.log("ERROR")
    } finally {
        console.log("clean up")
    }
}
console.log(zahl)
let test = "hihih"
test = "hi"
test = null
let x = 2
let y = 3
let testConst = 3
testConst = 4
switch(y) {
    case 3:
        console.log("3");
        break;
    default:
        console.log("unknown");
}
let unaryTest = false
if (!unaryTest) {
    console.log("unaryTest ist falsch")
}
if (((x == y) || (x != y))) {
    console.log(x, y)
}
for (let lol = 0; lol < 9; lol++) {
    console.log(lol)
}
let result = test(2, 4)
let stri = {str: "hi"}
let array1 = [1, 1, 2, 2, ]
result = array1[2]

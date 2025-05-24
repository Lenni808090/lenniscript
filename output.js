function test(num1, num2) {
    try {
        num1 += 1
        let res = (num1 + num2)
        return res
        for (let i = 0; (i < 10); ++i) {
            console.log("hi")
        }
    } catch {
        console.log("ERROR")
    } finally {
        console.log("clean up")
    }
}
let test = "hihih"
test = "hi"
test = null
let x = 2
let y = 3
switch(y) {
    case 3:
        console.log("3");
        break;
    default:
        console.log("unknown");
}
if (((x == y) || (x != y))) {
    console.log(x, y)
}
let result = test(2, 4)
let stri = {str: "hi"}
let array1 = [1, 1, 2, 2, ]
result = array1[2]

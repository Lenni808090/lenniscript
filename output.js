let x = {foo: "bar"};
function testfn(i) {
    while ((i < 5)) {
        if ((i == 4)) {
            x.foo = ["3", "bar", ];
        } else if((i == 5)) {
            x.foo = ["3", "apple", ];
        } else {
            x.foo = ["3", "grr", ];
        }
        i = (i + 1);
    }
}
console.log(x.foo[1])

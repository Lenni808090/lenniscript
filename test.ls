 let x = {
        foo: "bar",
        hi,
    };

    let i = 0;
     while(i < 5) {
        if(5 == 4) {
            x.foo = "3";
        }else if(4 == 5){
            x.foo = "2";
        }else {
            x.foo = "4";
        }
        i = i + 1;
        console.log(x.foo);
     }
     console.log(x.foo);
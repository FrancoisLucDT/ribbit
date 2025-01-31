pub mod rvm {
    use std::fmt::{Display, Formatter};
    use std::cmp::Ordering;
    use std::cmp::Ordering::Equal;
    use std::collections::HashMap;
    use std::io::*;
    use std::ops::{Add, Div, Mul, Sub};
    use std::process;


    // Data representation as Rib
//
// Pair: car,cdr,0  (Rib,Rib,0)
// Closure Procedure: code,env,1 (Rib,Rib,1)
//       code: nparams,0,start (int,0,Rib: Operation)
// Primitive Procedure: code,dontcare,1 ([0..19],0,1)
// Symbol: value,name,2 (Rib, Rib: String,2)
// String: chars,length,3 (Rib,int,3)
// Vector: elems,length,4 (Rib,int,4)
// #t,#f,(): dontcare,dontcare,5 (0,0,5)
    const PAIR: i32 = 0;
    const PROCEDURE: i32 = 1;
    const SYMBOL: i32 = 2;
    const STRING: i32 = 3;
    const VECTOR: i32 = 4;
    const SPECIAL: i32 = 5;


    // Operation representation as Rib
//
// jump: 0,slot/global,0 (0,Rib: Symbol, 0)
// call: 0,slot/global,next (0,Rib: Symbol,Rib: Operation)
// set: 1,slot/global,next(1,Rib: Symbol, Rib: Operation)
// get: 2,slot/global,next (2,Rib: Symbol, Rib: Operation)
// const: 3,object,next (3,Rib, Rib: Operation)
// if: 4,then,next (4,Rib: Operation, Rib: Operation)

    const CALL: i32 = 0;
    const SET: i32 = 1;
    const GET: i32 = 2;
    const CNST: i32 = 3;
    const IF: i32 = 4;
    const HALT: i32 = 5;




    // putchar

    fn putchar(c: char) {
        let mut stdo = stdout();
        let binding = c.to_string();
        let c_buffer =binding.as_bytes();
        stdo.write(c_buffer)
            .expect("Failed to write to stdo buffer");
        stdo.flush()
            .expect("Failed to flush stdo buffer");
    }

    fn decode_char_to_u32(c: Option<char>) -> u32 {
        match c {
            Some(ch) => ch as u32,
            None => panic!("Unexpected end of input"),
        }
    }


    //VM

    use std::ops::{Index, IndexMut};
    use std::str::{Chars, from_utf8};

    #[derive(Copy,Clone,PartialEq,Eq)]
    struct Rib {
        first: RibField,
        middle: RibField,
        last: RibField,
    }

    impl Display for Rib {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f,"[f:{},m:{},l:{}]",self.first.to_string(),
                   self.middle.to_string(),
                   self.last.to_string())
        }
    }





    #[derive(Copy,Clone,Eq)]
    enum RibField {
        Rib(usize),
        Number(i32),
    }

    impl Display for RibField {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match *self {
                RibField::Rib(ref inner) => write!(f,"r{}",*inner),
                RibField::Number(ref n) => write!(f,"n{}",*n),
            }
        }
    }

    impl PartialEq for RibField {
        fn eq(&self, other: &Self) -> bool {
            match self {
                RibField::Rib(ref inner) =>
                    match other {
                        RibField::Rib(ref other_inner) =>
                            inner == other_inner,
                        RibField::Number(_) => false,
                    },
                RibField::Number(ref n) =>
                    match other {
                        RibField::Rib(_) => false,
                        RibField::Number(ref other_n) => other_n == n,
                    }
            }
        }
    }

    impl PartialOrd for RibField {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            if self == other {Some(Equal)} else {
                match self {
                    RibField::Rib(_) => {
                        Option::None
                    },
                    RibField::Number(ref n) => {
                        match other {
                            RibField::Rib(_) => Option::None,
                            RibField::Number(ref other_n) => {
                                n.partial_cmp(other_n)
                            },
                        }
                    },
                }
            }

        }
    }

    impl Add for RibField{
        type Output = Option<RibField>;

        fn add(self, rhs: Self) -> Self::Output {
            match self {
                RibField::Number(ref n) => {
                    match rhs {
                        RibField::Number(ref m) => {
                            Some(RibField::Number(n + m))
                        },
                        _ => Option::None,
                    }
                },
                _ => Option::None,
            }
        }
    }

    impl Sub for RibField{
        type Output = Option<RibField>;

        fn sub(self, rhs: Self) -> Self::Output {
            match self {
                RibField::Number(ref n) => {
                    match rhs {
                        RibField::Number(ref m) => {
                            Some(RibField::Number(n - m))
                        },
                        _ => Option::None,
                    }
                },
                _ => Option::None,
            }
        }
    }

    impl Mul for RibField{
        type Output = Option<RibField>;

        fn mul(self, rhs: Self) -> Self::Output {
            match self {
                RibField::Number(ref n) => {
                    match rhs {
                        RibField::Number(ref m) => {
                            Some(RibField::Number(n * m))
                        },
                        _ => Option::None,
                    }
                },
                _ => Option::None,
            }
        }
    }

    impl Div for RibField {
        type Output = Option<RibField>;

        fn div(self, rhs: Self) -> Self::Output {
            match self {
                RibField::Number(ref n) => {
                    match rhs {
                        RibField::Number(ref m) => {
                            Some(RibField::Number(n / m))
                        },
                        _ => Option::None,
                    }
                },
                _ => Option::None,
            }
        }
    }

    impl RibField {
        fn get_rib(&self, holder: &mut RibHeap) -> Rib {
            match self {
                RibField::Rib(ref inner) => holder.get(inner),
                RibField::Number(n) =>
                    panic!("Expected a rib reference but got the number {}",n),
            }
        }

        fn get_number(&self) -> i32 {
            match self {
                RibField::Rib(ref inner) =>
                    {panic!("Expected a number but got the rib index {}",inner)},
                RibField::Number(ref n) => *n,
            }
        }

        fn get_rib_ref(&self) -> usize {
            match self {
                RibField::Rib(ref inner) => *inner,
                RibField::Number(ref n) =>
                    panic!("Expected a rib reference but got the number {}",n),
            }
        }

    }



    struct RibHeap {
        heap:Vec<Rib>,
    }

    impl RibHeap {
        fn push_rib(&mut self, data:Rib) -> usize {
            let index = self.heap.len(); // len() is how many ribs are before the pushed one
            self.heap.push(data);
            index
        }



        fn with_capacity(capacity: usize) -> Self {
            RibHeap{
                heap: Vec::with_capacity(capacity)
            }
        }

        fn set(&mut self, i:&usize, r:Rib) {
            self[*i] = r;
        }

        fn get(&mut self, i:&usize) -> Rib {
            self[*i]
        }

        fn garbage_collect(&mut self, stack: &mut usize, pc: &mut usize,symtbl: &mut usize) -> usize {

            let mut new_heap = Vec::with_capacity(self.heap.capacity());
            let mut index_correspondence:HashMap<usize,usize> = HashMap::new();

            new_heap.push(self.get(&0)); //FALSE
            new_heap.push(self.get(&1)); //TRUE
            new_heap.push(self.get(&2)); //NIL

            index_correspondence.insert(0,0);
            index_correspondence.insert(1,1);
            index_correspondence.insert(2,2);

            //Put every Rib referenced by a Rib in the stack, by a Rib in pc or by a Rib in the
            //symbol table in new_heap
            //Then, iterate through new_heap and change all RibField::Rib to their new index, as
            //recorded in index_correspondence


            if *symtbl<self.heap.len() {
                self.scan_and_sweep(symtbl, &mut new_heap, &mut index_correspondence);
            }

            if *pc<self.heap.len() {
                self.scan_and_sweep(pc, &mut new_heap, &mut index_correspondence);
            }

            if *stack<self.heap.len() {
                self.scan_and_sweep(stack, &mut new_heap, &mut index_correspondence);
            }


            let mut index: usize = 3;
            let impossible_ref =new_heap.len();

            while index<impossible_ref {
                let  rib_looked = new_heap.get(index).unwrap();
                let mut updated_rib = rib_looked.clone();
                let mut changed: bool =false;
                if is_rib(&rib_looked.first) {
                    updated_rib.first = RibField::Rib(
                        index_correspondence.get(&rib_looked.first.get_rib_ref()).unwrap().clone());
                    changed = true;
                }
                if is_rib(&rib_looked.middle) {
                    updated_rib.middle = RibField::Rib(
                        index_correspondence.get(&rib_looked.middle.get_rib_ref()).unwrap().clone());
                    changed = true;
                }
                if is_rib(&rib_looked.last) {
                    updated_rib.last = RibField::Rib(
                        index_correspondence.get(&rib_looked.last.get_rib_ref()).unwrap().clone());
                    changed =true;
                }
                if changed {
                    new_heap[index]= updated_rib;
                }
                index += 1;
            }
            self.heap = new_heap;
            index
        }

        fn scan_and_sweep(&mut self, start: &mut usize, new_heap: &mut Vec<Rib>,
                          index_correspondence: &mut HashMap<usize, usize>) {
            // ****
            // Contrat: Si le Rib à l'index_copied_rib est déjà dans le new_heap, par récursion,
            // les Ribs auxquels il est connexe sont déjà dedans
            // ****

            let mut list_ribs_to_copy=Vec::new();
            let mut index_copied_rib = *start;
            *start = if !index_correspondence.contains_key(start)
            {new_heap.len()}
            else
            {index_correspondence.get(start).unwrap().clone()};
            if *start != new_heap.len()
            {return;}
            list_ribs_to_copy.push(index_copied_rib);


            while !list_ribs_to_copy.is_empty() {
                if !index_correspondence.contains_key(&index_copied_rib)
                {
                    let copied_rib = self.get(&index_copied_rib);
                    Self::scan_for_copiable_rib_refs(&copied_rib, &mut list_ribs_to_copy,
                                                     &index_correspondence);
                    index_correspondence.insert(index_copied_rib, new_heap.len());
                    new_heap.push(copied_rib);
                };
                //Ne va jamais copier les trois premiers Ribs, car
                //scan_for_copiable_rib_refs peut ajouter au plus 3 éléments à list_ribs_to_copy,
                //qui doit contenir 4 ou plus éléments pour que la boucle soit exec
                // et pop n'enlève que 1 élément
                let next_rib = list_ribs_to_copy.pop();
                match next_rib {
                    Some(n) => index_copied_rib = n,
                    None => (),
                }
            }




        }

        // Adds Rib references to list if they aren't present
        fn scan_for_copiable_rib_refs(rib: &Rib, list: &mut Vec<usize>,
                                      index_correspondence: &HashMap<usize, usize>){
            match rib.first {
                RibField::Rib(ref inner) => {
                    if !index_correspondence.contains_key(inner)
                    {list.push(*inner)}
                },
                RibField::Number(_) => (),
            }
            match rib.middle {
                RibField::Rib(ref inner) => {
                    if !index_correspondence.contains_key(inner)
                    {list.push(*inner)}
                },
                RibField::Number(_) => (),
            }
            match rib.last {
                RibField::Rib(ref inner) => {
                    if !index_correspondence.contains_key(inner)
                    {list.push(*inner)}
                },
                RibField::Number(_) => (),
            }
        }
        //
    }

    impl Display for RibHeap{
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let mut record: String = String::new();

            let mut it = self.heap.iter();
            let mut current = it.next();
            let mut index: usize = 0;


            while current.is_some() {
                let current_str = *current.unwrap();
                let current_str = current_str.to_string();
                let mut entry_str = index.to_string();
                entry_str.push(':');
                entry_str.push_str(current_str.as_str());
                record.push_str(entry_str.as_str());
                record.push('\n');
                current = it.next();
                index += 1;
            }

            write!(f,"{}",record)
        }
    }

    impl Index<usize> for RibHeap{
        type Output = Rib;

        fn index(&self, index: usize) -> &Self::Output {
            &self.heap[index]
        }
    }

    impl IndexMut<usize> for RibHeap {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            &mut self.heap[index]
        }
    }


    const NIL: Rib = Rib {
        first: RibField::Number(0),
        middle: RibField::Number(0),
        last: RibField::Number(SPECIAL),
    };

    const NIL_REF:usize = 2;


    const TRUE: Rib = Rib {
        first: RibField::Number(0),
        middle: RibField::Number(0),
        last: RibField::Number(SPECIAL),
    };
    const TRUE_REF: usize = 1;

    const FALSE: Rib = Rib {
        first: RibField::Number(0),
        middle: RibField::Number(0),
        last: RibField::Number(SPECIAL),
    };

    const FALSE_REF: usize =0;

    fn make_rib(first: RibField, middle: RibField, last: RibField) -> Rib {
        Rib {
            first,
            middle,
            last
        }
    }

    fn make_data_rib(first: RibField, middle: RibField, last: i32) -> Rib {
        make_rib(first, middle, RibField::Number(last))
    }

    fn make_op_rib(first: i32, middle: RibField, last: RibField) -> Rib {
        make_rib(RibField::Number(first), middle, last)
    }






    fn show(o: &RibField, holder: &mut RibHeap) -> String{
        if !is_rib(o) {o.get_number().to_string()}
        else {
            let mut rib_o = o.get_rib(holder);
            let kind = rib_o.last;
            let mut result = String::new();
            match kind {
                RibField::Number(ref n) => match *n {
                    VECTOR => {result = String::from("#");
                        result.push_str(show(&rib_o.first,holder).as_str());
                    },
                    PAIR => { // Could also be tail call
                        let mut n =1;
                        result.push('(');
                        result.push_str(show(&rib_o.first, holder).as_str());
                        let mut o_middle = rib_o.middle;
                        while is_rib(&o_middle) &&
                            (!is_rib(&o_middle.get_rib(holder).last) &&
                                o_middle.get_rib(holder).last.get_number() == 0)
                        {
                            rib_o = o_middle.get_rib(holder);
                            if n > 4 {
                                result.push_str(" ...");
                                o_middle = RibField::Rib(NIL_REF);
                                break;
                            }
                            result.push(' ');
                            result.push_str(show(&rib_o.first, holder).as_str());
                            o_middle = rib_o.middle;
                            n += 1;
                        }
                        if o_middle != RibField::Rib(NIL_REF)
                        {
                            result.push_str(" . ");
                            result.push_str(show(&o_middle, holder).as_str());
                        }
                        result.push(')');
                    },
                    PROCEDURE => {
                        if is_rib(&rib_o.first) {
                            let rib_o_first = rib_o.first.get_rib(holder);
                            result.push_str("#<procedure nparams=");
                            result.push_str(rib_o_first.first.get_number().to_string().as_str());
                            result.push('>');
                        } else {
                            result.push_str("#<primitive ");
                            result.push_str(rib_o.first.get_number().to_string().as_str());
                            result.push('>');
                        }
                    },
                    SYMBOL => {
                        let mut field_o = rib_o.middle;
                        let mut cond = is_rib(&field_o);
                        if cond {
                            rib_o =field_o.get_rib(holder);
                            if (!is_rib(&rib_o.last) && rib_o.last.get_number() ==2) &&
                                (!is_rib(&rib_o.middle) && rib_o.middle.get_number() > 0)
                            {
                                field_o = rib_o.first;
                                while is_rib(&field_o) &&
                                    !is_rib(&field_o.get_rib(holder).last) &&
                                    field_o.get_rib(holder).last.get_number() == 0
                                {
                                    rib_o =field_o.get_rib(holder);
                                    let n =rib_o.first.get_number() as u32;
                                    let c = char::from_u32(n).unwrap();
                                    result.push(c);
                                    field_o = rib_o.middle;
                                }
                            }
                            else
                            { cond = false; }
                        }
                        if cond == false {
                            result.push_str("#<symbol ");
                            result.push_str(show(&field_o, holder).as_str());

                            result.push('>');
                        }
                    },
                    STRING => {
                        result.push('"');
                        let mut field_o = rib_o.first;

                        while is_rib(&field_o) && !is_rib(&field_o.get_rib(holder).last)
                            && field_o.get_rib(holder).last.get_number() == 0
                        {
                            rib_o = field_o.get_rib(holder);
                            let n = rib_o.first.get_number() as u32;
                            let mut c =char::from_u32(n).unwrap();
                            if c == '\n' {
                                c = 'n';
                                result.push_str("\\");
                            } else if c == '\r' {
                                c = 'r';
                                result.push('\\');
                            } else if c == '\t' {
                                c = 't';
                                result.push('\\');
                            } else if c == '\\' || c == '"' {
                                result.push('\\');
                            }
                            result.push(c);
                            field_o = rib_o.middle;
                        }
                        result.push('"');

                    },
                    SPECIAL => {
                        match o {
                            RibField::Rib(FALSE_REF) => result.push_str("#f"),
                            RibField::Rib(TRUE_REF) => result.push_str("#t"),
                            RibField::Rib(NIL_REF) => result.push_str("()"),
                            _ => {
                                result.push('[');
                                result.push_str(show(&rib_o.first, holder).as_str());
                                result.push(',');
                                result.push_str(show(&rib_o.middle, holder).as_str());
                                result.push(',');
                                result.push_str(show(&rib_o.last, holder).as_str());
                                result.push(']');
                            }
                        }
                    },
                    _ => {
                        result.push('[');
                        result.push_str(show(&rib_o.first, holder).as_str());
                        result.push(',');
                        result.push_str(show(&rib_o.middle, holder).as_str());
                        result.push(',');
                        result.push_str(show(&rib_o.last, holder).as_str());
                        result.push(']');
                    }
                },
                RibField::Rib(_) => {
                    result.push('[');
                    result.push_str(show(&rib_o.first, holder).as_str());
                    result.push(',');
                    result.push_str(show(&rib_o.middle, holder).as_str());
                    result.push(',');
                    result.push_str(show(&rib_o.last, holder).as_str());
                    result.push(']');
                }
            };
            result
        }
    }

    fn start_step(step_count: &mut u32, tracing: &mut bool, next_stamp: &mut u32,
                  start_tracing: &u32, stack: &usize, holder: &mut RibHeap) {
        *step_count += 1;
        if *step_count >= *start_tracing {
            *tracing = true;
        }
        if !*tracing {
            if *step_count >= *next_stamp
            {
                *next_stamp = f32::floor((*next_stamp as f32) *1.01 + 1.0) as u32;
                eprintln!("@{}",step_count.to_string());
            }
            return
        }
        let mut s = RibField::Rib(*stack);
        let mut rib_s = s.get_rib(holder);
        let mut result = String::new();
        result.push('@');
        result.push_str(step_count.to_string().as_str());
        result.push_str(" STACK = (");
        while !is_rib(&rib_s.last) && rib_s.last.get_number() == 0
        {
            result.push(' ');
            result.push_str(show(&rib_s.first,holder).as_str());
            s = rib_s.middle;
            if !is_rib(&s) {break;}
            rib_s = s.get_rib(holder);
        }
        result.push(')');
        eprintln!("{}",result);

    }



    fn is_rib(obj: &RibField) -> bool {
        match obj {
            RibField::Rib(_) => true,
            _ => false,
        }
    }




    fn to_bool<E>(expr: E) -> RibField where E: FnOnce() -> bool{
        if expr() { RibField::Rib(TRUE_REF)} else { RibField::Rib(FALSE_REF) }
    }





    //functions involving the stack

    fn push_stack(x: RibField, stack: &mut usize, holder:&mut RibHeap){
        *stack = holder.push_rib(make_data_rib(x,
                                               RibField::Rib(*stack),
                                               PAIR));
    }

    fn pop_stack(stack: &mut usize, holder: &mut RibHeap) ->RibField{
        let r = holder.get(&stack).first;
        *stack = holder.get(&stack).middle.get_rib_ref();
        r
    }

    fn rvm_getchar(stack: &mut usize, holder: &mut RibHeap) {
        let mut buf: [u8; 1] = [0; 1];
        stdin()
            .read(&mut buf)
            .expect("Failed to read character in standard input");
        let n = from_utf8(&buf).unwrap();
        let c =n.chars().next().unwrap();
        if c as i32 == 0
        {
            push_stack(RibField::Number(-1), stack, holder);
        } else {
        push_stack(RibField::Number(c as i32), stack, holder);
        }
    }


    fn rvm_prim1<F>(expected_nargs: u32, mut f: F,stack: &mut usize, holder: &mut RibHeap)
        where F: FnMut(RibField,&mut RibHeap) -> RibField{
        if expected_nargs != 1
        {
            incoherent_nargs_stop(expected_nargs,1,false)
        }
        let x =pop_stack(stack, holder);
        let r = f(x, holder);
        push_stack(
            r,
            stack, holder
        );
    }

    fn rvm_prim2<G>(expected_nargs: u32, mut f: G,stack: &mut usize, holder: &mut RibHeap)
        where G: FnMut(RibField,RibField, &mut RibHeap) -> RibField{
        if expected_nargs != 2
        {
            incoherent_nargs_stop(expected_nargs,2,false)
        }
        let x = pop_stack(stack, holder);
        let y = pop_stack(stack, holder);
        let r =f(x, y, holder);
        push_stack(r,
                   stack, holder
        );
    }

    fn rvm_prim3<H>(expected_nargs: u32, mut f: H,stack: &mut usize, holder: &mut RibHeap)
        where H: FnMut(RibField, RibField, RibField, &mut RibHeap) -> RibField{
        if expected_nargs != 3
        {
            incoherent_nargs_stop(expected_nargs,3,false)
        }
        let x = pop_stack(stack, holder);
        let y = pop_stack(stack, holder);
        let z = pop_stack(stack, holder);
        let r = f(x,y,z, holder);
        push_stack(r,
                   stack, holder
        );
    }

    fn rvm_arg2(stack: &mut usize, holder: &mut RibHeap){
        let x = pop_stack(stack, holder);
        pop_stack(stack, holder);
        push_stack(x, stack, holder);
    }

    fn rvm_close(stack: &mut usize, holder: &mut RibHeap){
        let f = pop_stack(stack,holder).get_rib(holder).first;
        let m = RibField::Rib(*stack);

        let closure = holder.push_rib(
            make_data_rib(f,
                          m,
                          PROCEDURE)
        );

        push_stack(RibField::Rib(closure),
                   stack, holder);
    }

    fn list_tail(list: &usize, i:u32, holder: &mut RibHeap) ->usize{
        if i==0 {*list} else {
            list_tail(&holder.get(list).middle.get_rib_ref(),
                      i-1, holder)
        }
    }
    // End of functions involving the stack

    fn get_byte(iter: &mut Chars)-> u32 {
        decode_char_to_u32(iter.next())
    }

    fn get_code(iter: &mut Chars)-> i32 {
        // donne un nombre entre 0 et 92
        // Le bytecode de Ribbit n'utilise pas ' ' (ASCII 32), '"' (ASCII 34), et '/' (ASCII 47)
        let x= get_byte(iter) as i32 -35 /*35: ASCII pour '#'*/ ;
        if x<0 {57 /*57: ASCII pour '9'*/} else {x}
    }

    fn get_int(mut n:i32,iter:&mut Chars) -> i32 {

        let x=get_code(iter); // x entre 0 et 92 inclusif
        n *= 46; /* 46= 92/2, ASCII pour '.' */
        if x<46 {
            n+x // n*46 + [0..45]
        } else {
            get_int(n+x-46,iter) // passe n*46 + [0..46] à get_int
        }
    }



    fn symbol_ref(n: u32, symtbl:&usize, holder: &mut RibHeap)-> usize {
        let tail_ref = list_tail(symtbl, n, holder);
        holder.get(&tail_ref).first.get_rib_ref()
    }

    fn get_opnd_ref(o: &RibField, stack: &usize , holder: &mut RibHeap) -> usize {
        match o {
            RibField::Rib(ref r) => *r,
            RibField::Number(ref n) => list_tail(stack, *n as u32, holder),
        }
    }

    fn get_opnd(o: &RibField, stack: &usize , holder: &mut RibHeap) -> Rib {
        let index = get_opnd_ref(o, stack, holder);
        holder.get(&index)
    }

    fn get_cont(stack: &usize, holder: &mut RibHeap) -> usize {
        let mut s = *stack;
        let mut s_last = holder.get(&s).last;
        while !is_rib(&s_last) {
            let s_middle =holder.get(&s).middle;
            s = s_middle.get_rib_ref();
            s_last = holder.get(&s).last;
        }
        s
    }

    fn set_global(val_ref:usize,symtbl:&mut usize,holder: &mut RibHeap) {
        let sym_top = holder.get(symtbl);
        let mut top_first = sym_top.first.get_rib(holder);
        top_first.first = RibField::Rib(val_ref);
        holder.set(&sym_top.first.get_rib_ref(), top_first);
        *symtbl = sym_top.middle.get_rib_ref();
    }

    fn incoherent_nargs_stop(nargs:u32,expected_nargs:u32, variadic:bool) {

        if variadic {
            eprintln!("Insufficient number of arguments. This function requires a minimum of {} arguments, got {}", expected_nargs, nargs);
            println!("Insufficient number of arguments. This function requires a minimum of {} arguments, got {}", expected_nargs, nargs);
        }
        else {
            eprintln!("Incorrect number of arguments. This function takes {} arguments, got {}", expected_nargs, nargs);
            println!("Incorrect number of arguments. This function takes {} arguments, got {}", expected_nargs, nargs);
        }
        process::exit(0x0100)
    }

    pub fn run_rvm() {

        let mut step_count:u32 =0;
        let start_tracing:u32 = 0;
        let mut next_stamp:u32 =0;
        let mut tracing = false;
        let heap_tracing = false;
        let mut debug = false;

        tracing = true;
        debug = true;

        // @@(replace ");'u?>vD?>vRD?>vRA?>vRA?>vR:?>vR=!(:lkm!':lkv6y" (encode 92)
        let rvm_code: String = ");'u?>vD?>vRD?>vRA?>vRA?>vR:?>vR=!(:lkm!':lkv6y".to_string();
        // )@@

        let mut pos = rvm_code.chars();

        let mut rib_heap: RibHeap = RibHeap::with_capacity(rvm_code.len());

        rib_heap.push_rib(FALSE);

        rib_heap.push_rib(TRUE);

        rib_heap.push_rib(NIL);

        let mut stack: usize;


        fn primitives(code:u8, expected_nargs: u32, mut stack: &mut usize, mut rib_heap: &mut RibHeap) {
            match code {
                0 =>
                    {
                        rvm_prim3(expected_nargs, |z, y, x, h| -> RibField
                            {
                                RibField::Rib(
                                    h.push_rib(
                                        make_rib(x, y, z)
                                    ))
                            },
                                  &mut stack, &mut rib_heap)
                    },
                1 =>
                    { rvm_prim1(expected_nargs,|x,_h|x,&mut stack,&mut rib_heap) },
                2 =>
                    { if expected_nargs != 2 {incoherent_nargs_stop(expected_nargs,2,false)}; (||->(){ pop_stack(&mut stack, &mut rib_heap);})();},
                3 =>
                    {if expected_nargs != 2 {incoherent_nargs_stop(expected_nargs,2,false)}; rvm_arg2(&mut stack, &mut rib_heap)},
                4 =>
                    {
                    if expected_nargs != 1 {incoherent_nargs_stop(expected_nargs, 1, false) };
                    rvm_close(&mut stack, &mut rib_heap)
                },
                5 =>
                    rvm_prim1(expected_nargs,|x, _h|
                                   to_bool(||is_rib(&x)),
                               &mut stack, &mut rib_heap),
                6 =>
                    rvm_prim1(expected_nargs,|x, h|x.get_rib(h).first,
                               &mut stack, &mut rib_heap),
                7 =>
                    rvm_prim1(expected_nargs,|x, h|x.get_rib(h).middle,
                               &mut stack, &mut rib_heap),
                8 =>
                    rvm_prim1(expected_nargs,|x,h|x.get_rib(h).last,
                               &mut stack, &mut rib_heap),
                9 =>
                    rvm_prim2(expected_nargs,|y,x, h|
                                   {let mut new_rib = x.get_rib(h);
                                       let x_index = x.get_rib_ref();
                                       new_rib.first=y;
                                       h.set(&x_index,new_rib);
                                       y},
                               &mut stack, &mut rib_heap),
                10 =>
                    rvm_prim2(expected_nargs,|y,x, h|
                                    {let mut new_rib = x.get_rib(h);
                                        let x_index = x.get_rib_ref();
                                        new_rib.middle=y;
                                        h.set(&x_index,new_rib);
                                        y},
                                &mut stack, &mut rib_heap),
                11 =>
                    rvm_prim2(expected_nargs,|y,x,h|
                                    {let mut new_rib = x.get_rib(h);
                                        let x_index = x.get_rib_ref();
                                        new_rib.last=y;
                                        h.set(&x_index,new_rib);
                                        y},
                                &mut stack, &mut rib_heap),
                12 =>
                    rvm_prim2(expected_nargs,|y, x,_h|
                                    { to_bool(||x==y)
                                    }, &mut stack, &mut rib_heap),
                13 =>
                    rvm_prim2(expected_nargs,|y, x,_h|
                                    { to_bool(||x<y)
                                    },
                                &mut stack, &mut rib_heap),
                14 =>
                    rvm_prim2(expected_nargs,|y, x, _h|
                                    { (x+y)
                                        .expect("Addition operands should both be numbers")
                                    },
                                &mut stack, &mut rib_heap),
                15 =>
                    rvm_prim2(expected_nargs,|y, x, _h|
                                    { (x-y)
                                        .expect("Subtraction operands should both be numbers")
                                    },
                                &mut stack, &mut rib_heap),
                16 =>
                    rvm_prim2(expected_nargs,|y, x, _h|
                                    { (x*y)
                                        .expect("Factors should both be numbers")
                                    },
                                &mut stack, &mut rib_heap),
                17 =>
                    rvm_prim2(expected_nargs,|y, x, _h|
                                    { match y {
                                        RibField::Number(0) => {println!("Division by zero");process::exit(1)}
                                        _ => ()
                                    };
                                        (x/y)
                                        .expect("Division operands should both be numbers")
                                    },
                                &mut stack, &mut rib_heap),
                18 =>
                    {
                    rvm_getchar(&mut stack, &mut rib_heap)
                },
                19 =>
                    rvm_prim1(expected_nargs,|x, _h| {
                    let n_to_push = x.get_number() as u32;
                    let c_to_write = char::from_u32(n_to_push)
                        .expect(format!("expected representable character, got {}",n_to_push)
                            .as_str());
                    putchar(c_to_write);
                    RibField::Number(n_to_push as i32)
                },
                                &mut stack, &mut rib_heap),
                20 =>
                    {
                    let mut n_elems = expected_nargs;
                    let mut elems = Vec::new();
                    while n_elems > 0 {
                        if !is_rib(&rib_heap.get(&stack).last) &&
                            rib_heap.get(&stack).last.get_number() == 0
                        {
                            elems.push(pop_stack(&mut stack, &mut rib_heap));
                            n_elems -= 1;
                        }
                        else
                        {
                            eprintln!("Expected {} elements in the list but stack had {} elements",
                                      expected_nargs, elems.len());
                            println!("Expected {} elements in the list but stack had {} elements",
                                     expected_nargs, elems.len());
                            process::exit(0x0100)
                        }
                    }

                    let mut new_list = NIL_REF;
                    for e in elems {
                        push_stack(e, &mut new_list, &mut rib_heap)
                    }
                    push_stack(RibField::Rib(new_list),&mut stack, &mut rib_heap);
                },
                21 =>
                    rvm_prim1(expected_nargs,|code, _h| {
                    match code {
                        RibField::Number(value) => process::exit(value),
                        RibField::Rib(_) => process::exit(0x0100),
                    }
                },
                                &mut stack, &mut rib_heap),
                n => panic!("Unexpected code for primitive call {n}"),
            }

        }

        // Build the initial symbol table

        let mut symtbl = NIL_REF;
        let mut n = get_int(0,&mut pos);
        // n = rvm_code[0]>=35?(rvm_code[0] -35), 57
        while n>0 /*si rvm_code[0]=='#', la boucle est skipped*/
        {
            //Ceci alloue des structures SYMBOL vides (noms= "", value= FALSE
            n -= 1;
            let inner = rib_heap.push_rib(make_data_rib(
                RibField::Rib(NIL_REF),
                RibField::Number(0),
                STRING));
            let outer = rib_heap.push_rib(make_data_rib(
                RibField::Rib(FALSE_REF),
                RibField::Rib(inner),
                SYMBOL,
            ));
            symtbl = rib_heap.push_rib(make_data_rib(
                RibField::Rib(outer),
                RibField::Rib(symtbl),
                PAIR
            ));
        };


        let mut accum = NIL_REF;
        let mut n=0;
        loop{
            let c = get_byte(&mut pos); // 1e iteration: c = rvm_code[1]
            if c==44 /*44: ASCII pour ','*/ {
                let inner = rib_heap.push_rib(make_data_rib(
                    RibField::Rib(accum),
                    RibField::Number(n),
                    STRING
                ));
                let outer = rib_heap.push_rib(make_data_rib(
                    RibField::Rib(FALSE_REF),
                    RibField::Rib(inner),
                    SYMBOL
                ));
                symtbl = rib_heap.push_rib(make_data_rib(
                    RibField::Rib(outer),
                    RibField::Rib(symtbl),
                    PAIR
                ));
                accum=NIL_REF;
                n=0;
            } else {
                if c==59 /*ASCII pour ';'*/ {break};
                let ch = c as i32;
                push_stack(RibField::Number(ch),&mut accum,&mut rib_heap);
                n+=1;
            }
        }

        let inner = rib_heap.push_rib(make_data_rib(
            RibField::Rib(accum),
            RibField::Number(n),
            STRING
        ));
        let outer = rib_heap.push_rib(make_data_rib(
            RibField::Rib(FALSE_REF),
            RibField::Rib(inner),
            SYMBOL
        ));
        symtbl = rib_heap.push_rib(make_data_rib(
            RibField::Rib(outer),
            RibField::Rib(symtbl),
            PAIR
        ));



        // Les procédures n'ont pas encore été construites ni assignées aux entrées de la symtbl

        // Decode the RVM instructions

        let mut n_field:RibField;

        stack = rib_heap.push_rib(make_data_rib(RibField::Number(6),RibField::Number(6),6));

        loop {
            let x = get_code(&mut pos); //1e iteration: 1e char après ';' dans rvm_code
            let mut n = x; // 0<=n<=92
            let mut d ;
            let mut op = CALL;
            loop{
                //
                // x<=22:op=CALL,  ??23=<x<=55:op=SET,
                // ??56=<x<=57:op=GET, ??58=<x<=60:op=CNST,
                // ??61<=x<=74:op=IF, ??75=<x<=81:op=HALT
                // 82<=x<=92 ???
                d = match op {
                    CALL => 20,
                    SET=> 30,
                    GET=> 0,
                    CNST=> 10,
                    IF=> 11,
                    HALT=> 4,
                    _ => panic!("Unexpected op value {}",op)
                };
                if n<= d+2 {break};
                n-=d+3;
                op+=1;

            };
            if x>90 {
                n_field=pop_stack(&mut stack,&mut rib_heap);
            } else {
                if op==CALL {
                    push_stack(RibField::Number(0),&mut stack, &mut rib_heap);
                    op+=1;
                };
                if n>=d { //n= d+2, d+1, ou d
                    if n==d {
                        n_field = RibField::Number(get_int(0,&mut pos));
                    } else {
                        n_field = RibField::Rib(symbol_ref(get_int(n-d-1,&mut pos) as u32, // n-d-1= 1, 0
                                                           &symtbl,&mut rib_heap));
                    }
                } else { // n < d
                    if op<CNST { //CALL, SET, GET
                        n_field = RibField::Rib(symbol_ref(n as u32,&symtbl,&mut rib_heap));
                    } else { //CNST, IF, HALT
                        n_field = RibField::Number(n);

                    }
                };
                if op>IF {
                    let popped = pop_stack(&mut stack,&mut rib_heap);
                    let inner = rib_heap.push_rib(make_rib(
                        n_field,
                        RibField::Number(0),
                        popped
                    ));
                    n_field = RibField::Rib(rib_heap.push_rib(make_data_rib(
                        RibField::Rib(inner),
                        RibField::Rib(NIL_REF),
                        PROCEDURE
                    )));
                    if !is_rib(&rib_heap.get(&stack).middle) {break};
                    op = IF;
                };
            };

            // Il ne fait que push des n0, ils sont modifiés ici
            let stack_first= rib_heap.get(&stack).first;
            let new_rib_ref = rib_heap.push_rib(
                make_op_rib(
                    op-1 as i32,
                    n_field,
                    stack_first
                ));
            let mut top_stack = rib_heap.get(&stack);
            top_stack.first = RibField::Rib(new_rib_ref);
            rib_heap.set(&stack, top_stack); // <- Là, spécifiquement
        };


        let n_first = n_field.get_rib(&mut rib_heap).first;
        let mut pc: RibField = n_first.get_rib(&mut rib_heap).last;


        set_global(rib_heap.push_rib(make_data_rib(RibField::Number(0),
                                                   RibField::Rib(symtbl),
                                                   PROCEDURE)),
                   &mut symtbl, &mut rib_heap);
        set_global(FALSE_REF,
                   &mut symtbl, &mut rib_heap);
        set_global(TRUE_REF,
                   &mut symtbl, &mut rib_heap);
        set_global(NIL_REF,
                   &mut symtbl, &mut rib_heap);

        let halt_instr = rib_heap.push_rib(make_op_rib(HALT,
                                                       RibField::Number(0),
                                                       RibField::Number(0)));

        let primordial_cont = make_op_rib(CALL,
                                          RibField::Number(0),
                                          RibField::Rib(halt_instr));

        stack = rib_heap.push_rib(primordial_cont);


        if tracing {
            eprintln!("{}",show(&pc,&mut rib_heap));
        }
        // let mut pc_trace = show(&pc, &mut rib_heap);
        // let mut stack_trace = show_stack(&stack, &mut rib_heap);
        // let mut getchar_calls = 0;

        let mut size_of_heap =rib_heap.heap.len();
        if heap_tracing {
            eprintln!("Heap size before first gc: {}", size_of_heap);
        }

        let mut pc_ref = pc.get_rib_ref();
        size_of_heap = rib_heap.garbage_collect(&mut stack, &mut pc_ref, &mut symtbl);
        pc = RibField::Rib(pc_ref);

        if heap_tracing {
            eprintln!("Heap size after first gc: {}", size_of_heap);
        }

        let mut gc_count: u32 = 1;

        loop{
            if debug {
                start_step(&mut step_count, &mut tracing, &mut next_stamp, &start_tracing , &stack, &mut rib_heap);
            } else {
                step_count += 1;
            }
            let mut o = pc.get_rib(&mut rib_heap).middle;
            let pc_instr = pc.get_rib(&mut rib_heap).first.get_number();
            match pc_instr {
                HALT => { if tracing {eprintln!("halt");}
                    return},
                // jump/call
                CALL => {
                    if tracing { if is_rib(&pc.get_rib(&mut rib_heap).last) {
                        eprintln!("call {}",show(&o,&mut rib_heap));
                    } else {eprintln!("jump {}",show(&o,&mut rib_heap));}
                    }
                    let mut nargs = pop_stack(&mut stack, &mut rib_heap).get_number();
                    let opnd_ref =get_opnd(&o, &stack, &mut rib_heap);
                    o = opnd_ref.first;
                    let mut c = o.get_rib(&mut rib_heap).first;
                    if is_rib(&c){ // c: code
                        let mut nparams = c.get_rib(&mut rib_heap)
                            .first.get_number();



                        /* Référence en C:
                            num vari = NUM(CAR(code))&1;
                            if ((!vari && nparams != nargs)||(vari && nparams > nargs)){
                                printf("*** Unexpected number of arguments nargs: %d nparams: %d vari: %b", nargs, nparams, vari);
                                exit(1);
                            }
                        */

                        let variadic = nparams % 2==1;
                        nparams = nparams >>1;

                        if !variadic && nparams != nargs || variadic && nparams > nargs
                        {
                            incoherent_nargs_stop(nargs as u32, nparams as u32, variadic);
                        }


                        let mut c2 = make_rib(RibField::Number(0),
                                              RibField::Rib(o.get_rib_ref()),
                                              RibField::Number(PAIR));
                        let mut s2 = rib_heap.push_rib(c2);
                        let c2_ref = s2;


                        /* Référence en C:
                            nargs-=nparams;
                            if (vari){
                            obj rest = NIL;
                            for(int i = 0; i < nargs; ++i){
                                rest = TAG_RIB(alloc_rib(pop(), rest, PAIR_TAG));
                            }
                            s2 = TAG_RIB(alloc_rib(rest, s2, PAIR_TAG));
                            }
                        */

                        nargs -= nparams;
                        if variadic
                        {
                            let mut rest = NIL_REF;
                            let mut i =0;
                            while i < nargs {
                                let arg =pop_stack(&mut stack, &mut rib_heap);
                                push_stack(arg, &mut rest, &mut rib_heap);
                                i -= 1;
                            }
                            push_stack(RibField::Rib(rest), &mut s2, &mut rib_heap);
                        }


                        while nparams >0{
                            let popped =pop_stack(&mut stack,&mut rib_heap);
                            push_stack(popped,&mut s2,&mut rib_heap);
                            nparams -=1;
                        };
                        if is_rib(&pc.get_rib(&mut rib_heap).last) {
                            //It's a call
                            c2.first=RibField::Rib(stack);
                            c2.last=pc.get_rib(&mut rib_heap).last;
                            rib_heap.set(&c2_ref,c2);
                        } else {
                            //It's a jump
                            let k = get_cont(&stack, &mut rib_heap);
                            c2.first=rib_heap.get(&k).first;
                            c2.last=rib_heap.get(&k).last;
                            rib_heap.set(&c2_ref,c2);
                        };

                        stack = s2;

                    } else {
                        primitives(c.get_number() as u8, nargs as u32, &mut stack, &mut rib_heap);
                        if is_rib(&pc.get_rib(&mut rib_heap).last)
                            || pc.get_rib(&mut rib_heap).last.get_number() !=0 {
                            //It's a call
                            c = pc;
                        } else {
                            //It's a jump
                            c= RibField::Rib(get_cont(&stack, &mut rib_heap));
                            let mut top_stack = rib_heap.get(&stack);
                            top_stack.middle = c.get_rib(&mut rib_heap).first;
                            rib_heap.set(&stack,top_stack);
                        }
                    }
                    pc = c.get_rib(&mut rib_heap).last;
                },
                SET => {
                    if tracing {eprintln!("set {}",show(&o, &mut rib_heap));}
                    let set_rib_index = get_opnd_ref(&o,&stack,&mut rib_heap);
                    let mut set_rib = rib_heap.get(&set_rib_index);
                    let top =pop_stack(&mut stack,&mut rib_heap);
                    set_rib.first = top;
                    rib_heap.set(&set_rib_index,set_rib);
                    pc = pc.get_rib(&mut rib_heap).last;
                },
                GET => {
                    if tracing {eprintln!("get {}",show(&o, &mut rib_heap));}
                    let opnd_ref =get_opnd(&o,&stack,&mut rib_heap);
                    let gotten_element =
                        opnd_ref.first;
                    push_stack(gotten_element,&mut stack, &mut rib_heap);
                    pc = pc.get_rib(&mut rib_heap).last;
                },
                CNST => {
                    if tracing {eprintln!("const {}",show(&o, &mut rib_heap));}
                    push_stack(o,&mut stack,&mut rib_heap);
                    pc = pc.get_rib(&mut rib_heap).last;
                },
                IF => {

                    let bool_expr = pop_stack(&mut stack, &mut rib_heap);
                    if tracing {eprintln!("if ({})",show(&bool_expr, &mut rib_heap));
                    }
                    if is_rib(&bool_expr) && bool_expr.get_rib_ref() == FALSE_REF
                    {
                        pc = pc.get_rib(&mut rib_heap).last;
                    } else {
                        pc = pc.get_rib(&mut rib_heap).middle;
                    };
                },
                _ => panic!("Unimplemented instruction number {}",pc_instr),
            };


            if 2*size_of_heap < rib_heap.heap.len() {
                gc_count += 1;
                if heap_tracing {
                    eprintln!("Heap size before {}th gc: {}", gc_count, size_of_heap);
                }
                pc_ref = pc.get_rib_ref();
                size_of_heap = rib_heap.garbage_collect(&mut stack,&mut pc_ref, &mut symtbl);
                pc = RibField::Rib(pc_ref);
                if heap_tracing {
                    eprintln!("Heap size after {}th gc: {}", gc_count, size_of_heap);
                }
            }
        }
    }
}

use self::rvm::run_rvm;

fn main() {
    run_rvm();
}




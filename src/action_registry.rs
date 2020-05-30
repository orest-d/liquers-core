use std::result::Result;

use crate::error::Error;
use crate::query::*;
use std::convert::TryInto;
use core::fmt::Display;
use std::ops::Fn;
use std::collections::HashMap;

pub trait CallableAction<T>{
    fn call_action(&self, input:T, arguments:&Vec<ActionParameter>) -> Result<T, Error>;
}
/*
impl<T,In,Out> CallableAction<T> for Fn(In)->Out
where T:TryInto<In>, Out:TryInto<T>{
    pub fn call(&self, input:T, arguments:Vec<ActionParameter>) -> Result<T, Error>{
        let f_input:In = input.try_into()
        .map_err(|e|
            Error::ConversionError{message:format!("Input argument conversion failed; {}",e)})?
        let out = self(f_input);
        let result:T = out.try_into()
        .map_err(|e|
            Error::ConversionError{message:format!("Result conversion failed; {}",e)})?
        Ok(result)
    }
}
*/

//pub struct Function1<In,Out>(Box<dyn FnMut(In) -> Out>);

impl<F,T> CallableAction<T> for F
where
    F:Fn(i32)->i32,
    T:TryInto<i32>,
    i32:TryInto<T>,
    <i32 as std::convert::TryInto<T>>::Error:Display,
    <T as std::convert::TryInto<i32>>::Error:Display
    {
    fn call_action(&self, input:T, _arguments:&Vec<ActionParameter>) -> Result<T, Error>{
        let f_input:i32 = input.try_into()
        .map_err(|e|
            Error::ConversionError{message:format!("Input argument conversion failed; {}",e)})?;

            let out:i32 = (*self)(f_input);
            let result:T = out.try_into()
            .map_err(|e|
                Error::ConversionError{message:format!("Result conversion failed; {}",e)})?;
                Ok(result)
    }
}

pub struct Function1<In,Out>(Box<dyn Fn(In)->Out>);
pub struct Function2<In1,In2,Out>(Box<dyn Fn(In1,In2)->Out>);
/*
fn call1<T,In,Out>(f:Function1<In,Out>,input:T)->Result<T, Error>
where
T:TryInto<In>,
<T as std::convert::TryInto<In>>::Error:Display,
Out:Into<T>
{
    let f_input:In = input.try_into()
    .map_err(|e|
        Error::ConversionError{message:format!("Input argument conversion failed; {}",e)})?;
    Ok(f.0(f_input).into())
}
*/
impl<T,In,Out> CallableAction<T> for Function1<In,Out>
where
    T:TryInto<In>,
    Out:Into<T>,
    <T as std::convert::TryInto<In>>::Error:Display
    {
    fn call_action(&self, input:T, _arguments:&Vec<ActionParameter>) -> Result<T, Error>{
        let f_input:In = input.try_into()
        .map_err(|e|
            Error::ConversionError{message:format!("Input argument conversion failed; {}",e)})?;

        let out:Out = self.0(f_input);
        let result:T = out.into();
        Ok(result)
    }
}

impl<T,In1,In2,Out> CallableAction<T> for Function2<In1,In2,Out>
where
    T:TryInto<In1>,
    In2: TryParameterFrom,
    Out:Into<T>,
    <T as std::convert::TryInto<In1>>::Error:Display
    {
    fn call_action(&self, input:T, arguments:&Vec<ActionParameter>) -> Result<T, Error>{
        let a1:In1 = input.try_into()
        .map_err(|e|
            Error::ConversionError{message:format!("Input argument conversion failed; {}",e)})?;
        let mut par = ActionParametersSlice(&arguments[..]);
        let a2:In2 =  par.try_parameters_into(&mut ())?;
        let out:Out = self.0(a1, a2);
        let result:T = out.into();
        Ok(result)
    }
}

pub struct HashMapActionRegistry<T>(
    HashMap<
        String,
        HashMap<String, Box<dyn CallableAction<T>>>
    >
);

impl<T> HashMapActionRegistry<T>{
    pub fn new()->Self{
        HashMapActionRegistry::<T>(HashMap::new())
    }

    pub fn register_callable_action(&mut self, ns:&str, name:&str, action:Box<dyn CallableAction<T>>){
        let ns = ns.to_owned();
        let name = name.to_owned();
        let ns_registry = self.0.entry(ns).or_insert(HashMap::new());
        ns_registry.insert(name, action);
    }

    pub fn call(&self, ns:&str, name:&str, input:T, arguments:&Vec<ActionParameter>)->Result<T, Error>{
        self.0.get(ns)
        .ok_or_else(|| Error::ActionNotRegistered{message:format!("Action {} not registered in namespace {}; no such namespace",name,ns)})
        .and_then(
            |ns_registry|
            ns_registry.get(name)
            .ok_or_else(|| Error::ActionNotRegistered{message:format!("Action {} not registered in namespace {}",name,ns)})
        )?.call_action(input, arguments)
    }
}

impl<T> Environment<T> for HashMapActionRegistry<T>{
    fn eval(&mut self, input:T, query:&str)->Result<T,Error>{
        let path = crate::parse::parse_query(query)?;

        let mut value = input;
        for action_request in path{
            value = self.call("root", &action_request.name, value, &action_request.parameters)?
        }
        Ok(value)
    }
}


#[cfg(test)]
mod tests{
    use super::*;
    use crate::value::*;

    #[test]
    fn closure_call_action()-> Result<(), Box<dyn std::error::Error>>{
        let a = |x:i32| x*x;
        let result = a.call_action(Value::Integer(2),&vec![])?;
        assert_eq!(result, Value::Integer(4));
        Ok(())
    }

    #[test]
    fn function1_call_action()-> Result<(), Box<dyn std::error::Error>>{
        let a = |x:i32| x*x;
        //let f:Function1<i32,i32> = Function1(Box::new(a));
        let result = Function1(Box::new(a)).call_action(Value::Integer(2),&vec![])?;
        assert_eq!(result, Value::Integer(4));
        Ok(())
    }
    #[test]
    fn function2_call_action()-> Result<(), Box<dyn std::error::Error>>{
        let a = |x:i32,y:i32| x*y;
        //let f:Function1<i32,i32> = Function1(Box::new(a));
        let result = Function2(Box::new(a)).call_action(Value::Integer(2),&vec![ActionParameter::new("3")])?;
        assert_eq!(result, Value::Integer(6));
        Ok(())
    }
    /*
    #[test]
    fn test3()-> Result<(), Box<dyn std::error::Error>>{
        let a = |x:i32| x*x;
        let f:Function1<i32,i32> = Function1(Box::new(a));
        assert_eq!(call1(f,Value::Integer(2))?, Value::Integer(4));
        Ok(())
    }
    */
    #[test]
    fn test_registry1()->Result<(),Box<dyn std::error::Error>>{
        let mut registry = HashMapActionRegistry::<Value>::new();
        let a = |x:i32| x*x;
        registry.register_callable_action("root", "test", Box::new(Function1(Box::new(a))));
        let result = registry.call("root", "test", Value::Integer(2), &vec![])?;
        assert_eq!(result, Value::Integer(4));
        Ok(())   
    }
    #[test]
    fn test_eval()->Result<(),Box<dyn std::error::Error>>{
        let mut registry = HashMapActionRegistry::<Value>::new();
        let square = |x:i32| x*x;
        let add = |x:i32,y:i32| x+y;
        registry.register_callable_action("root", "square", Box::new(Function1(Box::new(square))));
        registry.register_callable_action("root", "add", Box::new(Function2(Box::new(add))));
        let result = registry.eval(Value::Integer(2),"square/add-10")?;
        assert_eq!(result, Value::Integer(14));
        Ok(())   
    }
}
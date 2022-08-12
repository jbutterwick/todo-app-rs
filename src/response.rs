use item::*;
use output::*;
use todo::*;

enum ResponseType {
	EXIT,
	CONTINUE,
}

pub(crate) struct Response {
	kind: ResponseType,
}

impl Response {
	pub fn to_output(&self) -> Output {}
}

struct

// class StringResult extends Result {
// constructor(private readonly str: ColoredString) {
// super('continue')
// }
//
// toOuput(): Output {
// return this.str.asOutput()
// }
// }
//
// class Exit extends Result {
// toOuput(): Output {
// return new ColoredString('blue', 'bye!').asOutput()
// }
// }
//
// class ErrorResult extends StringResult {
// constructor(readonly error: string) {
// super(new ColoredString('red', error))
// }
// }
//
// export const help = new StringResult(
// new ColoredString(
// 'yellow',
// `
// Available commands:
// help                              Displays this help
// list                              Display the todo list
// add <todo item description>       Adds the item to the todo list
// done <todo item number>           Marks the item as done
// quit                              Exit the program`,
// ),
// )
//
// export class ListResult extends Result {
// constructor(private readonly items: Item[]) {
// super('continue')
// }
//
// toOuput(): Output {
// return to_output(this.items)
// }
// }
//
// export const emptyListHint = new StringResult(
// new ColoredString('yellow', 'List is empty.  Try adding some items'),
// )
//
// export const doneIndexError: Result = new ErrorResult(
// 'Done command must have a valid item index',
// )
//
// export const unexpectedArg: (cn: string) => Result = (commandName: string) =>
// new ErrorResult(`${commandName} command does not take any arguments`)
//
// export const missingArg: (cn: string) => Result = (commandName: string) =>
// new ErrorResult(`${commandName} command requires an argument`)
//
// export const unknown: Result = new ErrorResult(
// 'I do not understand your command.  ' + 'Enter help to display available commands.',
// )
//
// export const exit = new Exit('exit')
//
// export const next = async (
//     todo: Todo,
//     result: Result,
//     loop: (todo: Todo) => Promise<'done'>,
// stop: () => 'done',
// ): Promise<'done'> => {
// switch (result.kind) {
// case 'continue':
// return await loop(todo)
// case 'exit':
// return stop()
// }
// }

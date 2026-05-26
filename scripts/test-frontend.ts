import { runPrivacyTests } from "../ui/utils/privacy.ts";
import { AppError, toAppError } from "../ui/services/ipcResult.ts";

function runDoorServiceTests() {
  const assertAppError = (val: unknown, expectedMessage: string, testName: string) => {
    const res = toAppError(val);
    if (!(res instanceof AppError)) {
      throw new Error(`${testName} Failed: Expected result to be an instance of AppError`);
    }
    if (res.message !== expectedMessage) {
      throw new Error(
        `${testName} Failed: Expected message '${expectedMessage}', got '${res.message}'`
      );
    }
  };

  // Test 1: toAppError with an existing AppError
  const appErr = new AppError("Already an AppError");
  const res1 = toAppError(appErr);
  if (res1 !== appErr) {
    throw new Error("toAppError Test 1 Failed: Expected identical instance reference");
  }

  // Test 2: toAppError with standard built-in Error
  assertAppError(
    new Error("Standard built-in error message"),
    "Standard built-in error message",
    "Test 2"
  );

  // Test 3: toAppError with primitive string value
  assertAppError("Raw string error", "Raw string error", "Test 3");

  // Test 4: toAppError with an arbitrary object
  assertAppError({ foo: "bar" }, "[object Object]", "Test 4");

  // Test 5: toAppError with custom subclass of Error
  class CustomError extends Error {
    constructor(msg: string) {
      super(msg);
      this.name = "CustomError";
    }
  }
  assertAppError(new CustomError("My custom error"), "My custom error", "Test 5");

  // Test 6: toAppError with object containing .message property
  assertAppError(
    { message: "Object with message property" },
    "Object with message property",
    "Test 6"
  );

  // Test 7: toAppError with object containing .error property
  assertAppError({ error: "Object with error property" }, "Object with error property", "Test 7");

  // Test 8: toAppError with object containing non-string .message property
  assertAppError({ message: 999 }, "999", "Test 8");

  // Test 9: toAppError with object containing non-string .error property
  assertAppError({ error: true }, "true", "Test 9");

  // Test 10: toAppError with primitive number value
  assertAppError(500, "500", "Test 10");

  // Test 11: toAppError with primitive boolean value
  assertAppError(false, "false", "Test 11");

  // Test 12: toAppError with null
  assertAppError(null, "null", "Test 12");

  // Test 13: toAppError with undefined
  assertAppError(undefined, "undefined", "Test 13");

  // Test 14: toAppError with an array
  assertAppError(["error1", "error2"], "error1,error2", "Test 14");

  // Test 15: toAppError with standard Error with empty/missing message
  assertAppError(new Error(""), "Unknown Error", "Test 15");
}

try {
  runPrivacyTests();
  console.log("✓ All frontend privacy utility tests passed successfully!");
  runDoorServiceTests();
  console.log("✓ All doors/IPC error service utility tests passed successfully!");
  process.exit(0);
} catch (err) {
  console.error("Frontend utility self-test failed:", err);
  process.exit(1);
}

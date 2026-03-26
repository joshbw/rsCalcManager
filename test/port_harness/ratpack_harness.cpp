// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.
//
// C++ test harness for Ratpack functions.
// Reads JSON test cases from stdin, calls the C++ Ratpack functions,
// and outputs JSON results to stdout.
//
// Build: cmake -B build -G "Visual Studio 17 2022" -A x64
//        cmake --build build --config Release

#include "ratpak.h"

// Fix header/impl mismatch: sinrat actually takes 3 args in trans.cpp
extern void sinrat(PRAT* px, uint32_t radix, int32_t precision);

#include <iostream>
#include <string>
#include <sstream>
#include <cmath>
#include <map>
#include <functional>
#include <stdexcept>

// ---- Global state required by Ratpack ----
// Ratpack uses global variables for configuration.
// We initialize them here.
static bool g_initialized = false;

static void ensure_init(uint32_t radix, int32_t precision)
{
    // ChangeConstants initializes all Ratpack globals
    ChangeConstants(radix, precision);
    g_initialized = true;
}

// ---- Helpers ----

// Parse a simple rational string like "7", "-3", "1/2", "-5/3"
static PRAT parse_rational(const std::string& s, uint32_t radix, int32_t precision)
{
    // Check for "p/q" form
    auto slash = s.find('/');
    if (slash != std::string::npos) {
        std::string pstr = s.substr(0, slash);
        std::string qstr = s.substr(slash + 1);

        bool neg_p = false;
        if (!pstr.empty() && pstr[0] == '-') {
            neg_p = true;
            pstr = pstr.substr(1);
        }
        bool neg_q = false;
        if (!qstr.empty() && qstr[0] == '-') {
            neg_q = true;
            qstr = qstr.substr(1);
        }

        std::wstring wp(pstr.begin(), pstr.end());
        std::wstring wq(qstr.begin(), qstr.end());

        PRAT result = StringToRat(neg_p, wp, false, L"", radix, precision);
        PRAT denom = StringToRat(neg_q, wq, false, L"", radix, precision);

        // result = result / denom
        divrat(&result, denom, precision);
        destroyrat(denom);
        return result;
    }

    // Simple number (possibly with decimal point or exponent)
    bool neg = false;
    std::string num = s;
    if (!num.empty() && num[0] == '-') {
        neg = true;
        num = num.substr(1);
    }

    // Split on '.' for decimal
    auto dot = num.find('.');
    std::wstring mantissa;
    std::wstring exponent;

    // Check for 'e' or 'E' exponent
    auto epos = num.find_first_of("eE");
    if (epos != std::string::npos) {
        std::string mant_str = num.substr(0, epos);
        std::string exp_str = num.substr(epos + 1);

        // Remove dot from mantissa for StringToRat
        bool neg_exp = false;
        if (!exp_str.empty() && exp_str[0] == '-') {
            neg_exp = true;
            exp_str = exp_str.substr(1);
        } else if (!exp_str.empty() && exp_str[0] == '+') {
            exp_str = exp_str.substr(1);
        }

        // Combine mantissa parts
        auto mdot = mant_str.find('.');
        if (mdot != std::string::npos) {
            mant_str.erase(mdot, 1);
            // Adjust exponent for removed dot
            int adj = static_cast<int>(mant_str.length() - mdot);
            int orig_exp = std::stoi(exp_str);
            if (neg_exp) orig_exp = -orig_exp;
            orig_exp -= adj;
            neg_exp = (orig_exp < 0);
            exp_str = std::to_string(std::abs(orig_exp));
        }

        mantissa = std::wstring(mant_str.begin(), mant_str.end());
        exponent = std::wstring(exp_str.begin(), exp_str.end());
        return StringToRat(neg, mantissa, neg_exp, exponent, radix, precision);
    }

    if (dot != std::string::npos) {
        // Has decimal point - use StringToRat with adjusted exponent
        std::string whole = num.substr(0, dot);
        std::string frac = num.substr(dot + 1);
        std::string combined = whole + frac;
        int exp_val = -static_cast<int>(frac.length());

        mantissa = std::wstring(combined.begin(), combined.end());
        std::string exp_str = std::to_string(std::abs(exp_val));
        exponent = std::wstring(exp_str.begin(), exp_str.end());
        return StringToRat(neg, mantissa, (exp_val < 0), exponent, radix, precision);
    }

    // Plain integer
    mantissa = std::wstring(num.begin(), num.end());
    return StringToRat(neg, mantissa, false, L"", radix, precision);
}

// Format a rational to string
static std::string format_rational(PRAT rat, uint32_t radix, int32_t precision)
{
    std::wstring ws = RatToString(rat, NumberFormat::Float, radix, precision);
    return std::string(ws.begin(), ws.end());
}

// Escape a string for JSON output
static std::string json_escape(const std::string& s)
{
    std::string result;
    for (char c : s) {
        switch (c) {
            case '"':  result += "\\\""; break;
            case '\\': result += "\\\\"; break;
            case '\n': result += "\\n"; break;
            case '\r': result += "\\r"; break;
            case '\t': result += "\\t"; break;
            default:   result += c; break;
        }
    }
    return result;
}

// Simple JSON field extractor (no dependency on a JSON library)
static std::string json_get_string(const std::string& json, const std::string& key)
{
    std::string search = "\"" + key + "\"";
    auto pos = json.find(search);
    if (pos == std::string::npos) return "";

    pos = json.find(':', pos + search.length());
    if (pos == std::string::npos) return "";
    pos++;

    // Skip whitespace
    while (pos < json.length() && (json[pos] == ' ' || json[pos] == '\t')) pos++;

    if (pos < json.length() && json[pos] == '"') {
        // String value
        pos++;
        std::string result;
        while (pos < json.length() && json[pos] != '"') {
            if (json[pos] == '\\' && pos + 1 < json.length()) {
                pos++;
                switch (json[pos]) {
                    case '"':  result += '"'; break;
                    case '\\': result += '\\'; break;
                    case 'n':  result += '\n'; break;
                    default:   result += json[pos]; break;
                }
            } else {
                result += json[pos];
            }
            pos++;
        }
        return result;
    } else if (pos < json.length() && json[pos] == 'n') {
        return "null";
    } else {
        // Number or other value
        std::string result;
        while (pos < json.length() && json[pos] != ',' && json[pos] != '}' && json[pos] != ' ') {
            result += json[pos];
            pos++;
        }
        return result;
    }
}

static int json_get_int(const std::string& json, const std::string& key, int default_val)
{
    std::string val = json_get_string(json, key);
    if (val.empty() || val == "null") return default_val;
    try { return std::stoi(val); } catch (...) { return default_val; }
}

// Extract from "inputs" sub-object
static std::string json_get_input(const std::string& json, const std::string& key)
{
    auto inputs_pos = json.find("\"inputs\"");
    if (inputs_pos == std::string::npos) return "";
    auto brace = json.find('{', inputs_pos);
    if (brace == std::string::npos) return "";
    // Find matching closing brace
    int depth = 1;
    size_t end = brace + 1;
    while (end < json.length() && depth > 0) {
        if (json[end] == '{') depth++;
        else if (json[end] == '}') depth--;
        end++;
    }
    std::string inputs_json = json.substr(brace, end - brace);
    return json_get_string(inputs_json, key);
}

static int json_get_param(const std::string& json, const std::string& key, int default_val)
{
    auto params_pos = json.find("\"params\"");
    if (params_pos == std::string::npos) return default_val;
    auto brace = json.find('{', params_pos);
    if (brace == std::string::npos) return default_val;
    int depth = 1;
    size_t end = brace + 1;
    while (end < json.length() && depth > 0) {
        if (json[end] == '{') depth++;
        else if (json[end] == '}') depth--;
        end++;
    }
    std::string params_json = json.substr(brace, end - brace);
    std::string val = json_get_string(params_json, key);
    if (val.empty() || val == "null") return default_val;
    try { return std::stoi(val); } catch (...) { return default_val; }
}

// ---- Main harness ----

static void process_test_case(const std::string& line)
{
    std::string func = json_get_string(line, "function");
    std::string id = json_get_string(line, "id");
    uint32_t radix = static_cast<uint32_t>(json_get_param(line, "radix", 10));
    int32_t precision = static_cast<int32_t>(json_get_param(line, "precision", 128));

    ensure_init(radix, precision);

    std::string result_str;
    std::string error_str = "null";

    try {
        if (func == "add_rat") {
            PRAT a = parse_rational(json_get_input(line, "a"), radix, precision);
            PRAT b = parse_rational(json_get_input(line, "b"), radix, precision);
            addrat(&a, b, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a); destroyrat(b);
        }
        else if (func == "sub_rat") {
            PRAT a = parse_rational(json_get_input(line, "a"), radix, precision);
            PRAT b = parse_rational(json_get_input(line, "b"), radix, precision);
            subrat(&a, b, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a); destroyrat(b);
        }
        else if (func == "mul_rat") {
            PRAT a = parse_rational(json_get_input(line, "a"), radix, precision);
            PRAT b = parse_rational(json_get_input(line, "b"), radix, precision);
            mulrat(&a, b, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a); destroyrat(b);
        }
        else if (func == "div_rat") {
            PRAT a = parse_rational(json_get_input(line, "a"), radix, precision);
            PRAT b = parse_rational(json_get_input(line, "b"), radix, precision);
            divrat(&a, b, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a); destroyrat(b);
        }
        else if (func == "rem_rat") {
            PRAT a = parse_rational(json_get_input(line, "a"), radix, precision);
            PRAT b = parse_rational(json_get_input(line, "b"), radix, precision);
            remrat(&a, b);
            result_str = format_rational(a, radix, precision);
            destroyrat(a); destroyrat(b);
        }
        else if (func == "mod_rat") {
            PRAT a = parse_rational(json_get_input(line, "a"), radix, precision);
            PRAT b = parse_rational(json_get_input(line, "b"), radix, precision);
            modrat(&a, b);
            result_str = format_rational(a, radix, precision);
            destroyrat(a); destroyrat(b);
        }
        else if (func == "rat_pow_i32") {
            PRAT a = parse_rational(json_get_input(line, "base"), radix, precision);
            // 'power' is a JSON integer, not a string - use json_get_int helper
            int input_pos_start = line.find("\"inputs\"");
            std::string inputs_sub;
            if (input_pos_start != std::string::npos) {
                auto ib = line.find('{', input_pos_start);
                int d = 1; size_t ie = ib + 1;
                while (ie < line.length() && d > 0) {
                    if (line[ie] == '{') d++;
                    else if (line[ie] == '}') d--;
                    ie++;
                }
                inputs_sub = line.substr(ib, ie - ib);
            }
            int32_t exp_val = json_get_int(inputs_sub, "power", 0);
            ratpowi32(&a, exp_val, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "exp_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            exprat(&a, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "log_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            lograt(&a, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "log10_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            log10rat(&a, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "pow_rat") {
            PRAT base = parse_rational(json_get_input(line, "base"), radix, precision);
            PRAT exp = parse_rational(json_get_input(line, "exp"), radix, precision);
            powrat(&base, exp, radix, precision);
            result_str = format_rational(base, radix, precision);
            destroyrat(base); destroyrat(exp);
        }
        else if (func == "root_rat") {
            PRAT x = parse_rational(json_get_input(line, "x"), radix, precision);
            PRAT n = parse_rational(json_get_input(line, "n"), radix, precision);
            rootrat(&x, n, radix, precision);
            result_str = format_rational(x, radix, precision);
            destroyrat(x); destroyrat(n);
        }
        else if (func == "sin_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            sinrat(&a, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "cos_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            cosrat(&a, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "tan_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            tanrat(&a, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "sin_angle_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            std::string at = json_get_input(line, "angle_type");
            AngleType angle_type = AngleType::Radians;
            if (at == "Degrees") angle_type = AngleType::Degrees;
            else if (at == "Gradians") angle_type = AngleType::Gradians;
            sinanglerat(&a, angle_type, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "cos_angle_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            std::string at = json_get_input(line, "angle_type");
            AngleType angle_type = AngleType::Radians;
            if (at == "Degrees") angle_type = AngleType::Degrees;
            else if (at == "Gradians") angle_type = AngleType::Gradians;
            cosanglerat(&a, angle_type, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "tan_angle_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            std::string at = json_get_input(line, "angle_type");
            AngleType angle_type = AngleType::Radians;
            if (at == "Degrees") angle_type = AngleType::Degrees;
            else if (at == "Gradians") angle_type = AngleType::Gradians;
            tananglerat(&a, angle_type, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "asin_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            asinrat(&a, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "acos_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            acosrat(&a, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "atan_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            atanrat(&a, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "sinh_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            sinhrat(&a, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "cosh_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            coshrat(&a, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "tanh_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            tanhrat(&a, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "asinh_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            asinhrat(&a, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "acosh_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            acoshrat(&a, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "atanh_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            atanhrat(&a, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "fact_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            factrat(&a, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "and_rat") {
            PRAT a = parse_rational(json_get_input(line, "a"), radix, precision);
            PRAT b = parse_rational(json_get_input(line, "b"), radix, precision);
            andrat(&a, b, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a); destroyrat(b);
        }
        else if (func == "or_rat") {
            PRAT a = parse_rational(json_get_input(line, "a"), radix, precision);
            PRAT b = parse_rational(json_get_input(line, "b"), radix, precision);
            orrat(&a, b, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a); destroyrat(b);
        }
        else if (func == "xor_rat") {
            PRAT a = parse_rational(json_get_input(line, "a"), radix, precision);
            PRAT b = parse_rational(json_get_input(line, "b"), radix, precision);
            xorrat(&a, b, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a); destroyrat(b);
        }
        else if (func == "lsh_rat") {
            PRAT a = parse_rational(json_get_input(line, "a"), radix, precision);
            PRAT b = parse_rational(json_get_input(line, "b"), radix, precision);
            lshrat(&a, b, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a); destroyrat(b);
        }
        else if (func == "rsh_rat") {
            PRAT a = parse_rational(json_get_input(line, "a"), radix, precision);
            PRAT b = parse_rational(json_get_input(line, "b"), radix, precision);
            rshrat(&a, b, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a); destroyrat(b);
        }
        else if (func == "int_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            intrat(&a, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "frac_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            fracrat(&a, radix, precision);
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "rat_to_string") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            std::string fmt_str = json_get_input(line, "format");
            NumberFormat fmt = NumberFormat::Float;
            if (fmt_str == "Scientific") fmt = NumberFormat::Scientific;
            else if (fmt_str == "Engineering") fmt = NumberFormat::Engineering;
            std::wstring ws = RatToString(a, fmt, radix, precision);
            result_str = std::string(ws.begin(), ws.end());
            destroyrat(a);
        }
        else if (func == "string_to_number") {
            // Parse a string to number, then convert back to string for comparison
            std::string s = json_get_input(line, "s");
            std::wstring ws(s.begin(), s.end());
            PNUMBER pn = StringToNumber(ws, radix, precision);
            if (pn) {
                // Convert back via Rational for consistent output
                PRAT rat = nullptr;
                createrat(rat);
                rat->pp = pn;
                rat->pq = i32tonum(1, radix);
                result_str = format_rational(rat, radix, precision);
                destroyrat(rat);
            } else {
                error_str = "\"ParseError\"";
            }
        }
        else if (func == "string_to_rat") {
            std::string s = json_get_input(line, "s");
            std::wstring ws(s.begin(), s.end());
            // Handle mantissa/exponent parsing
            bool neg = false;
            if (!ws.empty() && ws[0] == L'-') { neg = true; ws = ws.substr(1); }
            PRAT rat = StringToRat(neg, ws, false, L"0", radix, precision);
            result_str = format_rational(rat, radix, precision);
            destroyrat(rat);
        }
        else if (func == "gcd_rat") {
            PRAT a = parse_rational(json_get_input(line, "x"), radix, precision);
            gcdrat(&a, precision);
            // After GCD, output p/q as "p/q" or just integer
            result_str = format_rational(a, radix, precision);
            destroyrat(a);
        }
        else if (func == "rat_equ") {
            PRAT a = parse_rational(json_get_input(line, "a"), radix, precision);
            PRAT b = parse_rational(json_get_input(line, "b"), radix, precision);
            bool eq = rat_equ(a, b, precision);
            result_str = eq ? "true" : "false";
            destroyrat(a); destroyrat(b);
        }
        else if (func == "rat_lt") {
            PRAT a = parse_rational(json_get_input(line, "a"), radix, precision);
            PRAT b = parse_rational(json_get_input(line, "b"), radix, precision);
            bool lt = rat_lt(a, b, precision);
            result_str = lt ? "true" : "false";
            destroyrat(a); destroyrat(b);
        }
        else if (func == "rat_gt") {
            PRAT a = parse_rational(json_get_input(line, "a"), radix, precision);
            PRAT b = parse_rational(json_get_input(line, "b"), radix, precision);
            bool gt = rat_gt(a, b, precision);
            result_str = gt ? "true" : "false";
            destroyrat(a); destroyrat(b);
        }
        else {
            error_str = "\"UNKNOWN_FUNCTION\"";
            result_str = "";
        }
    }
    catch (uint32_t err_code) {
        // Ratpack throws uint32_t error codes
        switch (err_code) {
            case CALC_E_DOMAIN:      error_str = "\"Domain\""; break;
            case CALC_E_INDEFINITE:  error_str = "\"Indefinite\""; break;
            case CALC_E_OVERFLOW:    error_str = "\"Overflow\""; break;
            case CALC_E_DIVIDEBYZERO: error_str = "\"DivideByZero\""; break;
            default:
                error_str = "\"CalcError_" + std::to_string(err_code) + "\"";
                break;
        }
        result_str = "";
    }
    catch (const std::exception& e) {
        error_str = "\"Exception: " + json_escape(e.what()) + "\"";
        result_str = "";
    }
    catch (...) {
        error_str = "\"UnknownException\"";
        result_str = "";
    }

    // Output JSON result
    std::cout << "{\"id\":\"" << json_escape(id)
              << "\",\"function\":\"" << json_escape(func)
              << "\",\"result\":";
    if (result_str.empty()) {
        std::cout << "null";
    } else {
        std::cout << "\"" << json_escape(result_str) << "\"";
    }
    std::cout << ",\"error\":" << error_str
              << "}" << std::endl;
}

int main(int argc, char* argv[])
{
    // Read JSON test cases from stdin, one per line
    std::string line;
    while (std::getline(std::cin, line)) {
        if (line.empty() || line[0] == '#') continue;  // Skip comments/empty lines
        process_test_case(line);
    }
    return 0;
}

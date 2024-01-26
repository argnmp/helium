const colors = require("tailwindcss/colors");
module.exports = {
    content: ["./src/**/*.{html,js}"],
    theme: {
        fontFamily: {
            post: ['-apple-system', 'BlinkMacSystemFont', "Apple SD Gothic Neo", "Pretendard Variable", 'Pretendard', 'Roboto', "Noto Sans KR", "Segoe UI", "Malgun Gothic", "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", 'sans-serif']
        },
        extend: {
            colors: {
                'customlight': colors.cyan,
                'customdark': colors.emerald,
            }
        }
    },
    plugins: [
        require('@tailwindcss/forms'),
    ],
    darkMode: ['class']
}

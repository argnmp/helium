const colors = require("tailwindcss/colors");
module.exports = {
    content: ["./src/**/*.{html,js}"],
    theme: {
        fontFamily: {
            post: ['-apple-system', 'BlinkMacSystemFont', "Apple SD Gothic Neo", "Pretendard Variable", 'Pretendard', 'Roboto', "Noto Sans KR", "Segoe UI", "Malgun Gothic", "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", 'sans-serif']
        },
        extend: {
            colors: {
                'customlight': {
                    '50': '#fef3f2',
                    '100': '#fee4e2',
                    '200': '#ffcec9',
                    '300': '#fdaba4',
                    '400': '#fa7b6f',
                    '500': '#f15142',
                    '600': '#e03d2e',
                    '700': '#bb281a',
                    '800': '#9b2419',
                    '900': '#80241c',
                    '950': '#460e09',
                },
                'customdark': {
                    '50': '#ffffe7',
                    '100': '#fdffc1',
                    '200': '#fffe86',
                    '300': '#fff541',
                    '400': '#ffe70d',
                    '500': '#ffd800',
                    '600': '#d19f00',
                    '700': '#a67202',
                    '800': '#89580a',
                    '900': '#74480f',
                    '950': '#442604',
                },

            }
        }
    },
    plugins: [],
    darkMode: ['class']
}

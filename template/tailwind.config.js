/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["./src/**/*.{html,js}"],
    theme: {
        fontFamily: {
            //post: ["애플 SD 산돌고딕 Neo","Apple SD Gothic Neo","나눔바른고딕",'NanumBarunGothic',"나눔고딕",'NanumGothic',"맑은 고딕","Malgun Gothic","돋움",'dotum','AppleGothic,sans-serif']
            //post: ['Inter var','ui-sans-serif','system-ui','-apple-system','BlinkMacSystemFont','Segoe UI','Roboto','Helvetica Neue','Arial','Noto Sans','sans-serif','Apple Color Emoji','Segoe UI Emoji','Segoe UI Symbol','Noto Color Emoji']
            post: ['-apple-system', 'BlinkMacSystemFont', "Apple SD Gothic Neo", "Pretendard Variable", 'Pretendard', 'Roboto', "Noto Sans KR", "Segoe UI", "Malgun Gothic", "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", 'sans-serif']
        }
    },
    plugins: [
        require('@tailwindcss/forms'),
    ],
}
